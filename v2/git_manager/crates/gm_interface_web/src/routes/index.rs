// crates/gm_interface_web/src/routes/index.rs
// The dashboard — the main landing page of the web interface.

use axum::{extract::State, response::Html};
use crate::{layout::{page, h, PageProps}, state::AppState};

/// GET / — renders the dashboard with account and repository summaries.
pub async fn dashboard(State(state): State<AppState>) -> Result<Html<String>, axum::http::StatusCode> {
    let accounts = state.services.services()
        .list_accounts(None).await
        .map_err(|e| { tracing::error!(error = %e, "dashboard: list_accounts failed"); axum::http::StatusCode::INTERNAL_SERVER_ERROR })?;
    let repos = state.services.services()
        .list_repositories(None).await
        .map_err(|e| { tracing::error!(error = %e, "dashboard: list_repositories failed"); axum::http::StatusCode::INTERNAL_SERVER_ERROR })?;

    let recent: String = if accounts.is_empty() {
        r#"<div class="empty-state" style="padding:var(--space-8) 0">
  <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
  <h2 class="empty-state-title">No accounts yet</h2>
  <p class="empty-state-description">Add a GitHub, GitLab, Bitbucket, or Azure DevOps account to get started.</p>
  <a href="/accounts/new" class="btn btn--primary">Add your first account</a>
</div>"#.to_string()
    } else {
        let rows: String = accounts.iter().take(8).map(|a| {
            let initials: String = a.alias.chars().take(2).collect::<String>().to_uppercase();
            let platform = a.platform_name.as_deref().unwrap_or("—");
            format!(
                r#"<tr>
  <td><div class="flex items-center gap-3">
    <div class="account-avatar" style="width:32px;height:32px;font-size:var(--text-sm)">{initials}</div>
    <div><div class="font-medium">{alias}</div><div class="text-xs text-muted">{email}</div></div>
  </div></td>
  <td><span class="badge badge--neutral text-xs">{platform}</span></td>
  <td class="font-mono text-sm">{username}</td>
  <td><a href="/accounts/{uuid}" class="btn btn--ghost btn--sm">View</a></td>
</tr>"#,
                initials = initials, alias = h(&a.alias), email = h(&a.email),
                platform = h(platform), username = h(&a.username), uuid = a.uuid,
            )
        }).collect();
        format!(r#"<div class="card"><div class="table-wrapper"><table>
  <thead><tr><th>Account</th><th>Platform</th><th>Username</th><th></th></tr></thead>
  <tbody>{rows}</tbody>
</table></div></div>"#)
    };

    let content = format!(r#"
<div class="page-header">
  <div>
    <h1 class="page-title">Dashboard</h1>
    <p class="page-subtitle">Overview of your Git accounts and repositories</p>
  </div>
</div>
<div class="page-body">
  <div class="grid-2" style="margin-bottom:var(--space-6)">
    <div class="card card--elevated">
      <div class="card-body">
        <div style="font-size:var(--text-3xl);font-weight:var(--weight-bold);color:var(--color-brand)">{acct_count}</div>
        <div class="text-sm text-muted" style="margin-top:var(--space-1)">Accounts</div>
      </div>
    </div>
    <div class="card card--elevated">
      <div class="card-body">
        <div style="font-size:var(--text-3xl);font-weight:var(--weight-bold);color:var(--color-brand)">{repo_count}</div>
        <div class="text-sm text-muted" style="margin-top:var(--space-1)">Repositories</div>
      </div>
    </div>
  </div>
  <h2 class="sidebar-section-title" style="margin-bottom:var(--space-3)">Recent Accounts</h2>
  {recent}
</div>"#,
        acct_count = accounts.len(),
        repo_count = repos.len(),
        recent = recent,
    );

    let breadcrumb = r#"<nav class="breadcrumb"><span class="breadcrumb-current">Dashboard</span></nav>"#;
    Ok(Html(page(PageProps { title: "Dashboard", active: "dashboard", breadcrumb, topbar_right: "", content: &content })))
}
