// crates/gm_interface_web/src/app.rs
//
// The Axum router and server bootstrap. This module owns the one thing the
// web interface uniquely needs: an HTTP listener. Everything else — state,
// services, routes — lives in separate modules and is composed here.
//
// ── Router composition ────────────────────────────────────────────────────────
// The router is built layer by layer. The browser routes (/, /accounts, /ssh,
// /repositories, /clone, /git-ops) serve HTML pages rendered by Askama
// templates. The API routes (/api/v1/...) serve JSON for the JavaScript
// components and for external tooling. The SSE endpoint (/api/sse) streams
// real-time events to connected browser tabs.
//
// Tower-HTTP middleware layers provide:
//   - CORS (permissive in dev, restricted in prod via environment variable)
//   - Request/response tracing for the structured logger
//   - Brotli compression for response bodies over 512 bytes
//
// ── Graceful shutdown ─────────────────────────────────────────────────────────
// The server listens for Ctrl-C and runs the kernel's shutdown lifecycle before
// closing connections. This ensures plugin on_unload() hooks fire and the
// database connection pool drains cleanly.

use std::sync::Arc;
use axum::{Router, routing::{get, post}, http::{HeaderValue, Method}};
use tower_http::{cors::CorsLayer, compression::CompressionLayer};

use gm_kernel::kernel::Kernel;
use gm_shared::errors::GitManagerError;

use crate::{
    routes::{accounts, clone, git_ops, index, repositories, ssh, static_files, api},
    services::WebServicesHandle,
    sse_bus::{SseBroadcastHandler, SseBus},
    state::AppState,
};

/// Starts the HTTP server and blocks until shutdown.
/// Called by `WebPlugin::run()`.
pub async fn run_server(kernel: Arc<Kernel>, addr: &str) -> Result<(), GitManagerError> {
    // The web services handle must be registered by the binary before this runs.
    let services = kernel
        .get::<WebServicesHandle>()
        .ok_or_else(|| GitManagerError::Other(
            "WebServicesHandle not in registry — check apps/web/main.rs bootstrap".to_string()
        ))?;

    // ── SSE broadcast bus ─────────────────────────────────────────────────────
    // Create the bus first, then hand its sender to the broadcast handler and
    // its subscribe() method to the AppState. Both share the same underlying
    // tokio::sync::broadcast channel via Arc<Sender>.
    let sse_bus = SseBus::new();
    let sse_handler = Arc::new(SseBroadcastHandler::new(&sse_bus));

    // Register the handler as a global subscriber so it receives ALL kernel
    // events regardless of type. This must happen before the server starts
    // accepting connections so no events are missed during the window between
    // server bind and the first SSE client connecting.
    kernel.event_bus.subscribe_all(sse_handler);

    tracing::debug!("SSE broadcast handler registered with kernel event bus");

    let state = AppState {
        kernel: Arc::clone(&kernel),
        services,
        sse_bus,
    };

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| GitManagerError::Other(format!("bind {addr}: {e}")))?;

    tracing::info!("Web interface listening on http://{addr}");
    tracing::info!("  Dashboard:  http://{addr}/");
    tracing::info!("  API docs:   http://{addr}/api/v1/");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| GitManagerError::Other(format!("server error: {e}")))?;

    tracing::info!("Web interface shut down");
    Ok(())
}

fn build_router(state: AppState) -> Router {
    // The browser UI sub-router handles HTML page routes.
    let ui_router = Router::new()
        .route("/",                    get(index::dashboard))
        .route("/accounts",            get(accounts::list_page))
        .route("/accounts/new",        get(accounts::add_page).post(accounts::add_submit))
        .route("/accounts/{uuid}",        get(accounts::detail_page))
        .route("/accounts/{uuid}/remove", post(accounts::remove_submit))
        .route("/repositories",           get(repositories::list_page))
        .route("/clone",                  get(clone::wizard_page).post(clone::wizard_submit))
        .route("/git-ops",                get(git_ops::ops_page))
        .route("/ssh",                    get(ssh::list_page))
        .route("/ssh/generate",           get(ssh::generate_page).post(ssh::generate_submit))
        .route("/ssh/{uuid}/test",        post(ssh::test_handler))
        .route("/static/{*path}",         get(static_files::serve_static));

    Router::new()
        // Flat single-level nesting avoids the double-nest root-path matching
        // issue in Axum 0.7 where "/" inside a doubly-nested router is silently
        // unreachable. All /api/v1/* routes are served by the v1 sub-router;
        // the SSE endpoint is registered at the top level separately.
        .nest("/api/v1", api::v1::router())
        .route("/api/sse", get(api::sse::sse_handler))
        .merge(ui_router)
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|req: &axum::http::Request<_>| {
                    tracing::debug_span!("http", method = %req.method(), uri = %req.uri())
                })
        )
        .layer(CompressionLayer::new())
        .layer(build_cors_layer())
        .with_state(state)
}

/// Builds the CORS layer from env var, falling back to permissive for development.
fn build_cors_layer() -> CorsLayer {
    match std::env::var("GIT_MANAGER_ALLOWED_ORIGIN") {
        Ok(origin) if !origin.is_empty() => {
            let origins: Vec<HeaderValue> = origin.split(',')
                .map(|s| s.trim().parse::<HeaderValue>().expect("Invalid CORS origin"))
                .collect();
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([Method::GET, Method::POST, Method::DELETE])
                .allow_headers([axum::http::header::CONTENT_TYPE])
        }
        _ => {
            tracing::debug!("GIT_MANAGER_ALLOWED_ORIGIN not set — using permissive CORS (dev mode)");
            CorsLayer::permissive()
        }
    }
}

/// Returns a future that resolves when Ctrl-C is pressed.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl-C signal handler");
    tracing::info!("Shutdown signal received");
}