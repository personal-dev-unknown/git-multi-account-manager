// Account management pages (HTML).
use axum::{extract::{Path, State}, response::{Html, Redirect}, Form};
use serde::Deserialize;
use uuid::Uuid;
use gm_ports::inbound::commands::AddAccountCommand;
use crate::{layout::{page, h, PageProps}, state::AppState};

pub async fn list_page(State(state): State<AppState>) -> Result<Html<String>, axum::http::StatusCode> {
    let accounts = state.services.services().list_accounts(None).await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let body = if accounts.is_empty() {
        r#"<div class="empty-state card">
  <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
    <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/>
  </svg>
  <h2 class="empty-state-title">No accounts yet</h2>
  <p class="empty-state-description">Add a GitHub, GitLab, Bitbucket, or Azure DevOps account to get started. Each account gets its own SSH key for complete isolation.</p>
  <a href="/accounts/new" class="btn btn--primary">Add your first account</a>
</div>"#.to_string()
    } else {
        let rows: String = accounts.iter().map(|a| {
            let initials: String = a.alias.chars().take(2).collect::<String>().to_uppercase();
            let status_class = match a.status.to_string().as_str() {
                s if s.contains("active") || s.contains("Active")   => "badge--success",
                s if s.contains("expired") || s.contains("Expired") => "badge--error",
                s if s.contains("unverified")                        => "badge--warning",
                _                                                     => "badge--neutral",
            };
            let default_badge = if a.is_default {
                r#" <span class="badge badge--info" style="font-size:10px;padding:1px 5px">default</span>"#
            } else { "" };
            format!(
                r#"<tr>
  <td><div class="flex items-center gap-3">
    <div class="account-avatar" style="width:32px;height:32px;font-size:var(--text-sm)">{initials}</div>
    <div>
      <div class="font-medium">{alias}{default_badge}</div>
      <div class="text-xs text-muted">{email}</div>
    </div>
  </div></td>
  <td><span class="badge badge--neutral">{platform}</span></td>
  <td class="font-mono text-sm">{username}</td>
  <td><span class="badge {status_class}">{status}</span></td>
  <td><div class="flex items-center gap-2">
    <a href="/accounts/{uuid}" class="btn btn--secondary btn--sm">View</a>
    <form method="POST" action="/accounts/{uuid}/remove" style="display:inline"
          onsubmit="return confirm('Remove account {alias}? This cannot be undone.')">
      <button type="submit" class="btn btn--danger btn--sm">Remove</button>
    </form>
  </div></td>
</tr>"#,
                uuid = a.uuid, alias = h(&a.alias), initials = initials, email = h(&a.email),
                platform = h(a.platform_name.as_deref().unwrap_or("—")),
                username = h(&a.username), status = h(&a.status.to_string()),
                status_class = status_class, default_badge = default_badge,
            )
        }).collect();
        format!(r#"<div class="card"><div class="table-wrapper"><table>
  <thead><tr><th>Account</th><th>Platform</th><th>Username</th><th>Status</th><th></th></tr></thead>
  <tbody>{rows}</tbody>
</table></div></div>"#)
    };

    let content = format!(r#"
<div class="page-header">
  <div>
    <h1 class="page-title">Accounts</h1>
    <p class="page-subtitle">Manage your Git hosting accounts and their SSH credentials</p>
  </div>
  <a href="/accounts/new" class="btn btn--primary">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><path d="M12 5v14M5 12h14"/></svg>
    Add Account
  </a>
</div>
<div class="page-body">{body}</div>"#);

    let breadcrumb = r#"<nav class="breadcrumb"><a href="/">Home</a><span class="breadcrumb-sep">/</span><span class="breadcrumb-current">Accounts</span></nav>"#;
    Ok(Html(page(PageProps { title: "Accounts", active: "accounts", breadcrumb, topbar_right: "", content: &content })))
}

pub async fn add_page(State(state): State<AppState>) -> Result<Html<String>, axum::http::StatusCode> {
    let platforms = state.services.services()
        .list_platforms()
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let platform_options: String = platforms.into_iter()
        .map(|p| {
            let hint = p.ssh_host
                .as_deref()
                .map(|h| format!(" ({})", h))
                .unwrap_or_default();
            format!("<option value=\"{}\">{}{}</option>", p.uuid, h(&p.display_name), hint)
        })
        .collect();

    let content = format!(r#"
<div class="page-header">
  <div>
    <h1 class="page-title">Add Account</h1>
    <p class="page-subtitle">Connect a Git hosting account. Each account gets an isolated SSH key.</p>
  </div>
</div>
<div class="page-body">
  <div class="card card--elevated" style="max-width:var(--content-narrow)">
    <div class="card-body">
      <form method="POST" action="/accounts/new" novalidate>
        <div class="form-group">
          <label class="form-label" for="platform_id">Git Hosting Platform <span style="color:var(--color-error)">*</span></label>
          <select id="platform_id" name="platform_id" class="select" required>
            <option value="">— Choose a platform —</option>
            {platform_options}
          </select>
        </div>
        <div class="form-group">
          <label class="form-label" for="alias">Account Alias <span style="color:var(--color-error)">*</span></label>
          <input type="text" id="alias" name="alias" class="input"
                 placeholder="e.g. work, personal, client-acme"
                 pattern="[a-z0-9][a-z0-9\\-]{{0,38}}" required maxlength="39">
          <p class="form-hint">Lowercase letters, numbers, and hyphens. Used in SSH key names and git config.</p>
        </div>
        <div class="form-group">
          <label class="form-label" for="username">Git Username <span style="color:var(--color-error)">*</span></label>
          <input type="text" id="username" name="username" class="input"
                 placeholder="e.g. shakamoses" required maxlength="100">
          <p class="form-hint">Your username on the hosting platform.</p>
        </div>
        <div class="form-group">
          <label class="form-label" for="email">Email Address <span style="color:var(--color-error)">*</span></label>
          <input type="email" id="email" name="email" class="input"
                 placeholder="you@example.com" required maxlength="254">
          <p class="form-hint">Used as the SSH key comment and git commit author email.</p>
        </div>
        <div class="form-group">
          <label class="form-label" for="auth_method">Authentication Method</label>
          <select id="auth_method" name="auth_method" class="select">
            <option value="ssh">SSH Key (recommended)</option>
            <option value="https_pat">Personal Access Token (PAT)</option>
          </select>
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <a href="/accounts" class="btn btn--secondary">Cancel</a>
          <button type="submit" class="btn btn--primary">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><path d="M12 5v14M5 12h14"/></svg>
            Add Account
          </button>
        </div>
      </form>
    </div>
  </div>
  <div class="alert alert--info mt-6" style="max-width:var(--content-narrow)">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18"><circle cx="12" cy="12" r="10"/><path d="M12 8v4M12 16h.01"/></svg>
    <div class="alert-body">
      <p class="alert-title">Next: Generate an SSH Key</p>
      After adding the account, go to <strong>SSH Keys</strong> to generate a key pair and add the public key to your hosting account.
    </div>
  </div>
</div>
<script>
  var authSelect = document.getElementById('auth_method');
  authSelect.addEventListener('change', function() {{
    document.getElementById('alias').focus();
  }});
</script>"#);
    let breadcrumb = r#"<nav class="breadcrumb"><a href="/">Home</a><span class="breadcrumb-sep">/</span><a href="/accounts">Accounts</a><span class="breadcrumb-sep">/</span><span class="breadcrumb-current">Add Account</span></nav>"#;
    Ok(Html(page(PageProps { title: "Add Account", active: "accounts", breadcrumb, topbar_right: "", content: &content })))
}

#[derive(Deserialize)]
pub struct AddAccountForm {
    pub alias: String, pub platform_id: String, pub username: String,
    pub email: String, pub auth_method: String,
}

pub async fn add_submit(
    State(state): State<AppState>,
    Form(form): Form<AddAccountForm>
) -> Result<Redirect, axum::http::StatusCode> {
    let platform_id = Uuid::parse_str(&form.platform_id)
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    let _ = state.services.services().add_account(AddAccountCommand {
        alias: form.alias, platform_id, username: form.username,
        email: form.email, auth_method: form.auth_method,
    }).await.map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    Ok(Redirect::to("/accounts"))
}

pub async fn detail_page(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<Html<String>, axum::http::StatusCode> {
    let account = state.services.services().get_account(uuid).await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    let initials: String = account.alias.chars().take(2).collect::<String>().to_uppercase();
    let default_badge = if account.is_default {
        r#" <span class="badge badge--info" style="margin-left:8px">default</span>"# } else { "" };
    let platform = h(account.platform_name.as_deref().unwrap_or("Unknown"));
    let alias = h(&account.alias);
    let uuid  = account.uuid;
    let content = format!(r#"
<div class="page-header">
  <div class="flex items-center gap-4">
    <div class="account-avatar">{initials}</div>
    <div>
      <h1 class="page-title">{alias}{default_badge}</h1>
      <p class="page-subtitle">{platform} · {email}</p>
    </div>
  </div>
</div>
<div class="page-body">
  <div class="card" style="max-width:600px">
    <table>
      <tbody>
        <tr><th style="width:160px">Platform</th><td>{platform}</td></tr>
        <tr><th>Username</th><td class="font-mono text-sm">{username}</td></tr>
        <tr><th>Email</th><td>{email}</td></tr>
        <tr><th>Auth Method</th><td><span class="badge badge--neutral">{auth}</span></td></tr>
        <tr><th>Status</th><td><span class="badge badge--success">{status}</span></td></tr>
        <tr><th>SSH Host Alias</th><td class="font-mono text-sm">{ssh}</td></tr>
        <tr><th>UUID</th><td class="font-mono text-xs">{uuid}</td></tr>
      </tbody>
    </table>
  </div>
  <div class="flex gap-3 mt-4">
    <a href="/ssh/generate?account={uuid}" class="btn btn--secondary">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>
      SSH Keys
    </a>
    <form method="POST" action="/accounts/{uuid}/remove"
          onsubmit="return confirm('Remove account {alias}? This cannot be undone.')">
      <button type="submit" class="btn btn--danger">Remove Account</button>
    </form>
  </div>
</div>"#,
        initials = initials, alias = alias, default_badge = default_badge, platform = platform,
        email = h(&account.email), username = h(&account.username),
        auth = h(&account.auth_method.to_string()), status = h(&account.status.to_string()),
        ssh = h(account.ssh_host_alias.as_deref().unwrap_or("—")),
        uuid = uuid,
    );
    let breadcrumb = format!(r#"<nav class="breadcrumb"><a href="/">Home</a><span class="breadcrumb-sep">/</span><a href="/accounts">Accounts</a><span class="breadcrumb-sep">/</span><span class="breadcrumb-current">{alias}</span></nav>"#);
    Ok(Html(page(PageProps { title: &alias, active: "accounts", breadcrumb: &breadcrumb, topbar_right: "", content: &content })))
}

pub async fn remove_submit(
    State(state): State<AppState>,
    Path(uuid): Path<Uuid>,
) -> Result<Redirect, axum::http::StatusCode> {
    state.services.services().remove_account(uuid).await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Redirect::to("/accounts"))
}
