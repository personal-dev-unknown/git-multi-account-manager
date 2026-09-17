use axum::{extract::{Path, State}, response::{Html, Redirect}, Form};
use serde::Deserialize;
use uuid::Uuid;
use gm_ports::inbound::commands::{GenerateSshKeyCommand, TestSshConnectionCommand};
use crate::{layout::{page, PageProps}, state::AppState};

pub async fn list_page(_state: State<AppState>) -> Result<Html<String>, axum::http::StatusCode> {
    let content = r#"
<div class="page-header">
  <div>
    <h1 class="page-title">SSH Keys</h1>
    <p class="page-subtitle">One active key per account — each key maps to a dedicated SSH config Host block</p>
  </div>
  <a href="/ssh/generate" class="btn btn--primary">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><path d="M12 5v14M5 12h14"/></svg>
    Generate Key
  </a>
</div>
<div class="page-body">
  <div class="empty-state card">
    <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/>
    </svg>
    <h2 class="empty-state-title">SSH Keys</h2>
    <p class="empty-state-description">SSH keys are managed per account. Go to an account to view or generate its key.</p>
    <a href="/accounts" class="btn btn--primary">Go to Accounts</a>
  </div>
</div>"#;
    let breadcrumb = r#"<nav class="breadcrumb"><a href="/">Home</a><span class="breadcrumb-sep">/</span><span class="breadcrumb-current">SSH Keys</span></nav>"#;
    Ok(Html(page(PageProps { title: "SSH Keys", active: "ssh", breadcrumb, topbar_right: "", content })))
}

pub async fn generate_page(_: State<AppState>) -> Result<Html<String>, axum::http::StatusCode> {
    let content = r#"
<div class="page-header">
  <div>
    <h1 class="page-title">Generate SSH Key</h1>
    <p class="page-subtitle">Create a new Ed25519 or RSA key pair for an account</p>
  </div>
</div>
<div class="page-body">
  <div class="card card--elevated" style="max-width:var(--content-narrow)">
    <div class="card-body">
      <form method="POST" action="/ssh/generate" novalidate>
        <div class="form-group">
          <label class="form-label" for="account_uuid">Account UUID</label>
          <input id="account_uuid" name="account_uuid" class="input input--mono"
                 placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" required>
          <p class="form-hint">Find this on the <a href="/accounts">account detail page</a></p>
        </div>
        <div class="form-group">
          <label class="form-label" for="key_type">Key Type</label>
          <select id="key_type" name="key_type" class="select">
            <option value="ed25519" selected>Ed25519 (recommended)</option>
            <option value="rsa">RSA 4096</option>
            <option value="ecdsa">ECDSA P-256</option>
            <option value="dsa" disabled>DSA (deprecated — not supported)</option>
          </select>
          <p class="form-hint">Ed25519 is preferred. RSA and ECDSA are supported. DSA is disabled (deprecated by OpenSSH).</p>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <a href="/ssh" class="btn btn--secondary">Cancel</a>
          <button type="submit" class="btn btn--primary">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><path d="M12 5v14M5 12h14"/></svg>
            Generate Key
          </button>
        </div>
      </form>
    </div>
  </div>
</div>"#;
    let breadcrumb = r#"<nav class="breadcrumb"><a href="/">Home</a><span class="breadcrumb-sep">/</span><a href="/ssh">SSH Keys</a><span class="breadcrumb-sep">/</span><span class="breadcrumb-current">Generate</span></nav>"#;
    Ok(Html(page(PageProps { title: "Generate SSH Key", active: "ssh", breadcrumb, topbar_right: "", content })))
}

#[derive(Deserialize)]
pub struct GenerateKeyForm { pub account_uuid: Uuid, pub key_type: String }

pub async fn generate_submit(
    State(state): State<AppState>,
    Form(form): Form<GenerateKeyForm>,
) -> Result<Redirect, axum::http::StatusCode> {
    state.services.services().generate_ssh_key(GenerateSshKeyCommand {
        account_uuid: form.account_uuid, key_type: form.key_type,
        comment: None, passphrase: None, add_to_agent: true,
    }).await.map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Redirect::to("/ssh"))
}

pub async fn test_handler(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<axum::Json<serde_json::Value>, axum::http::StatusCode> {
    let result = state.services.services()
        .test_ssh_connection(TestSshConnectionCommand { account_uuid: uuid, timeout_ms: Some(10_000) })
        .await.map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(axum::Json(serde_json::json!({ "success": result.success, "username": result.username })))
}
