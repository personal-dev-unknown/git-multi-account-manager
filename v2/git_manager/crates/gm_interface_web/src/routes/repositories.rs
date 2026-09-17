use axum::{extract::State, response::Html};
use crate::{layout::{page, h, PageProps}, state::AppState};

pub async fn list_page(State(state): State<AppState>) -> Result<Html<String>, axum::http::StatusCode> {
    let repos = state.services.services().list_repositories(None).await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let body = if repos.is_empty() {
        r#"<div class="empty-state card">
  <svg class="empty-state-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 3h18M3 9h18M3 15h18M3 21h18"/></svg>
  <h2 class="empty-state-title">No Repositories</h2>
  <p class="empty-state-description">Clone your first repository to get started.</p>
  <a href="/clone" class="btn btn--primary">Clone a Repository</a>
</div>"#.to_string()
    } else {
        let rows: String = repos.iter().map(|r| {
            let status = if r.is_cloned {
                "<span class=\"badge badge--success\">Cloned</span>"
            } else {
                "<span class=\"badge badge--neutral\">Not cloned</span>"
            };
            format!(
                "<tr><td class=\"font-mono text-sm\">{}</td><td class=\"font-mono text-xs\">{}</td><td>{status}</td><td><a href=\"/clone\" class=\"btn btn--secondary btn--sm\">Clone</a></td></tr>",
                h(&r.full_name), h(&r.default_branch),
            )
        }).collect();
        format!(r#"<div class="card"><div class="table-wrapper"><table>
  <thead><tr><th>Repository</th><th>Default Branch</th><th>Status</th><th></th></tr></thead>
  <tbody>{rows}</tbody>
</table></div></div>"#)
    };

    let content = format!(r#"
<div class="page-header">
  <div>
    <h1 class="page-title">Repositories</h1>
    <p class="page-subtitle">All repositories known to Git Manager</p>
  </div>
</div>
<div class="page-body">{body}</div>"#);

    let breadcrumb = r#"<nav class="breadcrumb"><a href="/">Home</a><span class="breadcrumb-sep">/</span><span class="breadcrumb-current">Repositories</span></nav>"#;
    let topbar_right = r#"<a href="/clone" class="btn btn--secondary btn--sm">
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
  Clone Repo
</a>"#;
    Ok(Html(page(PageProps { title: "Repositories", active: "repositories", breadcrumb, topbar_right, content: &content })))
}
