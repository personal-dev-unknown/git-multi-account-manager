// crates/gm_interface_web/src/routes/api/v1/docs.rs
//
// API discovery endpoint — GET /api/v1/
//
// This module owns the self-describing index handler for the v1 REST API.
// It intentionally carries no application state dependency: the endpoint
// is a pure, allocation-light constant-fold of a static JSON document that
// is built once and returned verbatim on every call.
//
// Architectural role
// ──────────────────
// Sits at the interface layer only — no ports, no domain, no kernel access.
// Its single responsibility is discoverability: a developer or operator
// hitting /api/v1/ receives a machine-readable map of every resource the
// API exposes, including its HTTP method, path, and purpose.
//
// This follows the REST HATEOAS spirit without introducing hypermedia
// complexity: the index is a stable, versioned contract document, not a
// dynamic link graph. Updating it when a new endpoint is added is
// intentional and visible in version control diffs.

use axum::response::Json;

/// Machine-readable discovery document for the v1 REST API.
///
/// Returns a static JSON object describing every registered endpoint.
/// The document is versioned (`"version": "v1"`) so consumers can assert
/// compatibility before issuing further requests.
///
/// # Concurrency
/// Pure function — no shared state, no locks, no async I/O.
/// Axum may call this from any Tokio worker thread concurrently;
/// there is no contention risk.
///
/// # Performance
/// `serde_json::json!` macro expands at compile time into a typed
/// `Value` tree. The only runtime cost is serialising the constant
/// tree into the response body once per request — O(n) in the size
/// of the document, typically < 1 µs.
pub async fn index() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "api":     "git-zyrix",
        "version": "v1",
        "base":    "/api/v1",
        "endpoints": {
            "accounts": [
                { "method": "GET",    "path": "/api/v1/accounts",               "description": "List all registered accounts" },
                { "method": "POST",   "path": "/api/v1/accounts",               "description": "Register a new account" },
                { "method": "GET",    "path": "/api/v1/accounts/{uuid}",        "description": "Retrieve a single account by UUID" },
                { "method": "DELETE", "path": "/api/v1/accounts/{uuid}",        "description": "Remove an account and its associated data" },
                { "method": "POST",   "path": "/api/v1/accounts/{uuid}/default","description": "Mark an account as the platform default" }
            ],
            "ssh_keys": [
                { "method": "GET",  "path": "/api/v1/ssh-keys",             "description": "List all SSH keys" },
                { "method": "POST", "path": "/api/v1/ssh-keys/generate",    "description": "Generate a new SSH key pair for an account" },
                { "method": "POST", "path": "/api/v1/ssh-keys/{uuid}/test", "description": "Test whether an SSH key authenticates with its platform" }
            ],
            "repositories": [
                { "method": "GET",  "path": "/api/v1/repositories",       "description": "List locally-cloned repositories" },
                { "method": "POST", "path": "/api/v1/repositories/clone", "description": "Clone a remote repository using an account SSH key" }
            ],
            "git": [
                { "method": "POST", "path": "/api/v1/git/pull",   "description": "Pull remote changes into a local repository" },
                { "method": "POST", "path": "/api/v1/git/push",   "description": "Stage, commit, and push local changes" },
                { "method": "GET",  "path": "/api/v1/git/status", "description": "Return working-directory status for a repository" }
            ]
        }
    }))
}
