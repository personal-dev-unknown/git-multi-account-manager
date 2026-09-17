// crates/gm_interface_web/src/state.rs
// Axum application state — shared across all request handlers.

use std::sync::Arc;
use gm_kernel::kernel::Kernel;
use crate::services::WebServicesHandle;
use crate::sse_bus::SseBus;

/// The Axum application state passed via `State<AppState>` to every handler.
/// Clone is cheap: `Arc` fields increment a reference count, `SseBus` clones
/// its inner `Arc<broadcast::Sender>`. No heap allocation occurs on clone.
#[derive(Clone)]
pub struct AppState {
    /// The kernel — used for event publishing and service registry access.
    pub kernel: Arc<Kernel>,
    /// The type-erased application services facade for this interface.
    /// Registered by apps/web/main.rs before the web plugin starts.
    pub services: Arc<WebServicesHandle>,
    /// Broadcast bus used by the SSE endpoint to stream kernel events to
    /// all connected browser clients. Cloning is reference-count-only.
    pub sse_bus: SseBus,
}