pub mod accounts;
pub mod docs;
pub mod git_ops;
pub mod repositories;
pub mod ssh;

use axum::{Router, routing::{get, post}};
use crate::state::AppState;

/// Assembles the /api/v1 sub-router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/",                        get(docs::index))
        // Account endpoints
        .route("/accounts",               get(accounts::list).post(accounts::create))
        .route("/accounts/{uuid}",         get(accounts::get_one).delete(accounts::remove))
        .route("/accounts/{uuid}/default", post(accounts::set_default))
        // SSH endpoints
        .route("/ssh-keys",               get(ssh::list))
        .route("/ssh-keys/generate",      post(ssh::generate))
        .route("/ssh-keys/{uuid}/test",    post(ssh::test_connection))
        // Repository endpoints
        .route("/repositories",           get(repositories::list))
        .route("/repositories/clone",     post(repositories::clone_repo))
        .route("/repositories/remote",    get(repositories::list_remote))
        // Git operation endpoints
        .route("/git/pull",               post(git_ops::pull))
        .route("/git/push",               post(git_ops::push))
        .route("/git/status",             get(git_ops::status))
}
