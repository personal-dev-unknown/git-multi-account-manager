use axum::{extract::{Path, State}, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use gm_ports::inbound::commands::{GenerateSshKeyCommand, TestSshConnectionCommand};
use gm_shared::models::ssh_key::SshKeyDto;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<SshKeyDto>>, StatusCode> {
    state.services.services().list_ssh_keys(Uuid::nil()).await
        .map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Deserialize)]
pub struct GenerateKeyRequest { pub account_uuid: Uuid, pub key_type: String, pub add_to_agent: bool }

pub async fn generate(
    State(state): State<AppState>,
    Json(req): Json<GenerateKeyRequest>,
) -> Result<(StatusCode, Json<SshKeyDto>), StatusCode> {
    let key = state.services.services()
        .generate_ssh_key(GenerateSshKeyCommand {
            account_uuid: req.account_uuid, key_type: req.key_type,
            comment: None, passphrase: None, add_to_agent: req.add_to_agent,
        }).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(key)))
}

#[derive(Serialize)]
pub struct TestResult { pub success: bool, pub username: Option<String>, pub error: Option<String> }

pub async fn test_connection(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<TestResult>, StatusCode> {
    let r = state.services.services()
        .test_ssh_connection(TestSshConnectionCommand { account_uuid: uuid, timeout_ms: Some(10_000) })
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(TestResult { success: r.success, username: r.username, error: r.error }))
}
