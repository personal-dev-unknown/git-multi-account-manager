use axum::{extract::State, Json, http::StatusCode};
use serde::Deserialize;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct PullRequest { pub repo_path: String }

/// Pull is not yet exposed on WebServices — returns 501 until the trait grows
/// git_pull/git_push methods.
pub async fn pull(
    State(_): State<AppState>,
    Json(_req): Json<PullRequest>,
) -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

#[derive(Deserialize)]
pub struct PushRequest { pub repo_path: String, pub message: String }

pub async fn push(
    State(_): State<AppState>,
    Json(_req): Json<PushRequest>,
) -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

pub async fn status(_: State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"entries": [], "is_clean": true}))
}
