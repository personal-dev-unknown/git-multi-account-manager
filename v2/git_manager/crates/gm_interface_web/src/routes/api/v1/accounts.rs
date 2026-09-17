// /api/v1/accounts — JSON REST endpoints for account management.
use axum::{extract::{Path, State}, Json, http::StatusCode};
use serde::Deserialize;
use uuid::Uuid;
use gm_ports::inbound::commands::AddAccountCommand;
use gm_shared::models::account::AccountDto;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<AccountDto>>, StatusCode> {
    state.services.services().list_accounts(None).await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_one(State(state): State<AppState>, Path(uuid): Path<Uuid>) -> Result<Json<AccountDto>, StatusCode> {
    state.services.services().get_account(uuid).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
pub struct CreateAccountRequest {
    pub alias: String, pub platform: String, pub username: String,
    pub email: String, pub auth_method: String,
}

pub async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<AccountDto>), StatusCode> {
    let platform_id = crate::routes::platform_uuid_for(&req.platform);
    let account = state.services.services()
        .add_account(AddAccountCommand {
            alias: req.alias, platform_id, username: req.username,
            email: req.email, auth_method: req.auth_method,
        })
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok((StatusCode::CREATED, Json(account)))
}

pub async fn remove(State(state): State<AppState>, Path(uuid): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    state.services.services().remove_account(uuid).await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn set_default(State(state): State<AppState>, Path(uuid): Path<Uuid>) -> Result<StatusCode, StatusCode> {
    let acc = state.services.services().get_account(uuid).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    state.services.services().set_default_account(acc.uuid, acc.platform_id).await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
