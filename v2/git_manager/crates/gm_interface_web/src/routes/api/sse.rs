// crates/gm_interface_web/src/routes/api/sse.rs
//
// Production Server-Sent Events endpoint.
//
// ── Data flow ─────────────────────────────────────────────────────────────────
//
//  Kernel domain service                     Browser EventSource
//       │                                           │
//       │ kernel.publish(SomeEvent)                 │
//       ▼                                           │
//  EventBus::dispatch()                             │
//       │                                           │
//       ▼                                           │
//  SseBroadcastHandler::handle()             SSE long-poll HTTP/1.1
//       │                                           │
//       │ broadcast::Sender::send(Arc<evt>)         │
//       ▼                                           ▼
//  broadcast ring buffer ──── BroadcastStream ──► sse_handler stream
//                                                   │
//                                          Event { id, event, data }
//                                                   │
//                                       browser dispatches to listeners
//                                        EventSource.addEventListener(...)
//
// ── Resumability ─────────────────────────────────────────────────────────────
// The SSE `id` field on every event is set to the `KernelEvent::event_id` UUID.
// The browser's `EventSource` stores the most recently seen id as `lastEventId`
// and sends it as the `Last-Event-ID` header on reconnect. This handler uses
// that header to detect reconnects and immediately emits a `resync` event,
// directing the client to re-fetch full state via the REST API. We cannot replay
// missed events from the broadcast ring buffer — it is not a persistent log.
//
// ── Lag / backpressure ────────────────────────────────────────────────────────
// If a receiver's cursor falls more than BROADCAST_CAPACITY events behind the
// sender's tail, `BroadcastStream` surfaces a `Lagged(n)` error. The handler
// converts this into a `resync` event and continues streaming. The connection
// is NOT closed on lag — only a full TCP failure or client disconnect ends it.
//
// ── Keep-alive ────────────────────────────────────────────────────────────────
// Axum injects keep-alive comment frames every 30 s. This prevents reverse
// proxies (nginx, caddy, AWS ALB) from timing out idle SSE connections.
//
// ── Security surface ─────────────────────────────────────────────────────────
// The endpoint carries no session credentials. Authentication is enforced by
// the tower middleware stack (CorsLayer) which restricts origin. The payload is
// the full KernelEvent JSON envelope — callers must not publish events that
// contain cleartext secrets in their payload; use opaque UUIDs instead.
//
// ── Concurrency model ─────────────────────────────────────────────────────────
// Each call to `sse_handler` subscribes a new independent `Receiver`. All
// receivers share the same ring buffer via `Arc<broadcast::Sender>`. The stream
// future runs on Axum's tokio runtime and is fully `Send + 'static`. Dropping
// the future (client disconnect) automatically unregisters the receiver.

use std::{convert::Infallible, time::Duration};

use axum::{
    extract::State,
    http::HeaderMap,
    response::sse::{Event, KeepAlive, Sse},
};
use chrono::Utc;
use tokio_stream::{
    wrappers::{errors::BroadcastStreamRecvError, BroadcastStream},
    StreamExt,
};
use uuid::Uuid;

use gm_kernel::contracts::event::KernelEvent;

use crate::state::AppState;

// ── Event type constants ───────────────────────────────────────────────────────

/// Emitted once immediately after the SSE connection is established.
/// Carries the current kernel sequence number so the client can compare it
/// with the sequence in `Last-Event-ID` on its next reconnect.
const EVENT_CONNECTED: &str = "connected";

/// Emitted when the client must re-fetch full state from the REST API.
/// Two triggers: (1) client reconnects after a gap, (2) receiver lag overflow.
const EVENT_RESYNC: &str = "resync";

// ── Frame construction helpers ────────────────────────────────────────────────

/// Converts a `KernelEvent` envelope into a fully-populated SSE frame.
///
/// SSE wire format produced:
/// ```text
/// id: <event_id UUID>
/// event: <event_type, e.g. "AccountAdded">
/// data: <JSON-serialised full KernelEvent envelope>
/// ```
///
/// Using `event_id` as the SSE `id` satisfies the SSE spec requirement that
/// ids are unique per stream and enables the browser to correctly report
/// `lastEventId` on reconnect.
///
/// The complete `KernelEvent` JSON is sent as data. This gives clients access
/// to the typed `payload`, `correlation_id`, `sequence`, and `timestamp_utc`
/// in one frame. Clients interested only in the event type can read the `event`
/// field without parsing `data` at all.
///
/// Serialisation failure is handled defensively: a minimal error JSON is
/// returned rather than panicking or dropping the frame silently.
fn kernel_event_to_sse(event: &KernelEvent) -> Result<Event, Infallible> {
    let data = match serde_json::to_string(event) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(
                event_id   = %event.event_id,
                event_type = %event.event_type,
                error      = %e,
                "sse: KernelEvent serialisation failed — emitting error frame"
            );
            format!(
                r#"{{"error":"serialisation_failed","event_id":"{}","event_type":"{}"}}"#,
                event.event_id, event.event_type
            )
        }
    };

    Ok(Event::default()
        .id(event.event_id.to_string())
        .event(event.event_type.clone())
        .data(data))
}

/// Builds a `resync` SSE frame with a structured JSON payload.
///
/// The `reason` field distinguishes why a resync was triggered:
/// - `"reconnected"` — client sent `Last-Event-ID`, meaning it missed events.
/// - `"lagged"` — receiver ring-buffer cursor fell too far behind.
///
/// `detail` is merged into the payload JSON object to carry context-specific
/// fields (e.g. `missed_events` count for lag, `current_sequence` for reconnect).
fn resync_frame(reason: &str, detail: serde_json::Value) -> Result<Event, Infallible> {
    let mut payload = serde_json::json!({
        "reason":    reason,
        "timestamp": Utc::now().to_rfc3339(),
    });

    if let (Some(p), Some(d)) = (payload.as_object_mut(), detail.as_object()) {
        for (k, v) in d {
            p.insert(k.clone(), v.clone());
        }
    }

    let data = serde_json::to_string(&payload)
        .unwrap_or_else(|_| format!(r#"{{"reason":"{}"}}"#, reason));

    Ok(Event::default()
        .id(Uuid::new_v4().to_string())
        .event(EVENT_RESYNC)
        .data(data))
}

/// Builds the initial `connected` SSE frame sent immediately on connection.
///
/// `sequence` is the kernel's current event sequence counter at the moment the
/// SSE handler runs. The client stores this value and, on its next reconnect,
/// compares it with the sequence embedded in the `Last-Event-ID` to determine
/// whether it missed events. If the two differ, the client should also trigger
/// a resync refetch even if the server does not emit one (defensive).
fn connected_frame(sequence: u64) -> Result<Event, Infallible> {
    let data = serde_json::to_string(&serde_json::json!({
        "sequence":  sequence,
        "timestamp": Utc::now().to_rfc3339(),
    }))
    .unwrap_or_else(|_| format!(r#"{{"sequence":{}}}"#, sequence));

    Ok(Event::default()
        .id(Uuid::new_v4().to_string())
        .event(EVENT_CONNECTED)
        .data(data))
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `GET /api/sse` — upgrades the HTTP connection to a Server-Sent Events stream.
///
/// # Connection lifecycle
///
/// 1. **Subscribe**: A new `broadcast::Receiver` is created from `AppState::sse_bus`.
///    Events published *before* this call are not visible (no replay); only
///    events published *after* are delivered.
///
/// 2. **Initial frames**: The `connected` event is always sent first. If the
///    client is reconnecting (`Last-Event-ID` header present), a `resync` event
///    follows, directing the client to refetch via REST API.
///
/// 3. **Live stream**: Each `KernelEvent` from the broadcast channel is mapped
///    to an SSE frame and flushed to the client. The stream runs until the
///    client disconnects (TCP close / navigation away) or the server shuts down.
///
/// 4. **Lag recovery**: If the client falls more than `BROADCAST_CAPACITY`
///    events behind, a `resync` frame is emitted and streaming continues.
///    The connection is never forcibly closed due to lag alone.
///
/// 5. **Keep-alive**: A comment-based ping is sent every 30 seconds to
///    prevent proxy timeouts on idle connections.
///
/// # Client-side usage
/// ```javascript
/// const es = new EventSource('/api/sse');
///
/// es.addEventListener('connected', e => {
///     const { sequence } = JSON.parse(e.data);
///     console.log('SSE connected, kernel sequence:', sequence);
/// });
///
/// es.addEventListener('AccountAdded', e => {
///     const event = JSON.parse(e.data);  // full KernelEvent envelope
///     store.addAccount(event.payload);
/// });
///
/// es.addEventListener('resync', () => {
///     // Missed events — refetch all state from REST API
///     fetchAllState();
/// });
/// ```
pub async fn sse_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl axum::response::IntoResponse {
    // ── Detect reconnect ───────────────────────────────────────────────────────
    // The browser's EventSource sends `Last-Event-ID: <uuid>` on reconnect.
    // We use its presence as a signal that the client missed events, without
    // needing to parse or validate the UUID value itself (we cannot replay
    // missed events from the broadcast ring buffer anyway).
    let reconnecting  = headers.contains_key("Last-Event-ID");
    let current_seq   = state.kernel.event_bus.current_sequence();

    // Subscribe *before* logging so receiver_count reflects this connection.
    let rx = state.sse_bus.subscribe();

    tracing::debug!(
        reconnecting       = reconnecting,
        current_sequence   = current_seq,
        active_connections = state.sse_bus.receiver_count(),
        "SSE connection established"
    );

    // ── Initial frames ─────────────────────────────────────────────────────────
    // Sent immediately before the live broadcast stream begins. Using a Vec
    // allows push-without-branching; tokio_stream::iter wraps it into a stream.
    let mut preamble: Vec<Result<Event, Infallible>> = Vec::with_capacity(2);

    // Always emit `connected` so the client records the current sequence.
    preamble.push(connected_frame(current_seq));

    // On reconnect, the client may have missed events we cannot replay.
    // Emitting `resync` is the correct recovery action.
    if reconnecting {
        preamble.push(resync_frame(
            "reconnected",
            serde_json::json!({
                "current_sequence": current_seq,
                "message": "Missed events are not replayable — refetch state via REST API",
            }),
        ));
    }

    // ── Live broadcast stream ──────────────────────────────────────────────────
    // `BroadcastStream` wraps the `Receiver` and implements `Stream`. Each item
    // is either `Ok(Arc<KernelEvent>)` or `Err(BroadcastStreamRecvError::Lagged(n))`.
    // The `.map()` converts both variants into `Result<Event, Infallible>`, which
    // is the item type required by `Sse::new()`.
    let live_stream = BroadcastStream::new(rx).map(move |result| {
        match result {
            Ok(event) => kernel_event_to_sse(&event),

            Err(BroadcastStreamRecvError::Lagged(missed)) => {
                tracing::warn!(
                    missed_events = missed,
                    "SSE receiver lagged — broadcasting resync event to client"
                );
                resync_frame(
                    "lagged",
                    serde_json::json!({
                        "missed_events": missed,
                        "message": "Ring-buffer overflow — refetch state via REST API",
                    }),
                )
            }
        }
    });

    // ── Compose full stream ────────────────────────────────────────────────────
    // Preamble items are yielded synchronously (no await), then the live
    // broadcast stream runs indefinitely until the client disconnects.
    let full_stream = tokio_stream::iter(preamble).chain(live_stream);

    // ── Keep-alive ─────────────────────────────────────────────────────────────
    // Axum injects `": keep-alive\n\n"` comment frames every 30 seconds.
    // Comments do not fire any EventSource event listener — they are purely
    // a transport-level mechanism to keep the TCP connection alive through
    // proxies, load balancers, and firewalls that close idle connections.
    Sse::new(full_stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    )
}
