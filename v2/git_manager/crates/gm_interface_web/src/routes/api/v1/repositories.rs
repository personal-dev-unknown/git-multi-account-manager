use axum::{extract::{Query, State}, Json, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;
use gm_ports::inbound::commands::CloneRepositoryCommand;
use gm_shared::models::repository::RepositoryDto;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<RepositoryDto>>, StatusCode> {
    state.services.services().list_repositories(None).await
        .map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Deserialize)]
pub struct CloneRequest { pub account_uuid: uuid::Uuid, pub url: String, pub dest_path: Option<String> }

pub async fn clone_repo(
    State(state): State<AppState>,
    Json(req): Json<CloneRequest>,
) -> Result<(StatusCode, Json<RepositoryDto>), StatusCode> {
    let repo = state.services.services()
        .clone_repository(CloneRepositoryCommand {
            account_uuid: Some(req.account_uuid),
            url:          req.url,
            destination:  req.dest_path,
            ..Default::default()
        }).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(repo)))
}

#[derive(Deserialize)]
pub struct ListRemoteQuery {
    pub account_uuid: Uuid,
    pub page:         Option<u32>,
    pub per_page:     Option<u32>,
}

pub async fn list_remote(
    State(state): State<AppState>,
    Query(q): Query<ListRemoteQuery>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let page     = q.page.unwrap_or(1);
    let per_page = q.per_page.unwrap_or(15);
    state.services.services()
        .list_remote_repositories(q.account_uuid, page, per_page)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
