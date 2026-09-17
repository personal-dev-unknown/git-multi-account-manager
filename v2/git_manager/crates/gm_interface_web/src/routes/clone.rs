use axum::{extract::{Query, State}, response::{Html, Redirect}, Form};
use serde::Deserialize;
use gm_ports::inbound::commands::CloneRepositoryCommand;
use crate::{layout::{page, h, PageProps}, state::AppState};

#[derive(Deserialize, Default)]
pub struct ClonePageQuery { pub error: Option<String> }

pub async fn wizard_page(
    State(state): State<AppState>,
    Query(q): Query<ClonePageQuery>,
) -> Html<String> {
    let accounts = state.services.services()
        .list_accounts(None).await
        .unwrap_or_default();

    let options: String = accounts.iter().map(|a| {
        format!(r#"<option value="{alias}">{alias} ({platform})</option>"#,
            alias    = h(&a.alias),
            platform = a.platform_id)
    }).collect();

    let no_accounts_hint = if accounts.is_empty() {
        r#"<p class="form-hint" style="color:var(--color-warning)">
             No accounts found. <a href="/accounts/new">Add an account</a> first.
           </p>"#
    } else {
        r#"<p class="form-hint">Select the account whose SSH key will be used for cloning</p>"#
    };

    let error_banner = match &q.error {
        Some(msg) => format!(
            r#"<div class="alert alert--error" style="margin-bottom:1rem;padding:.75rem 1rem;border-radius:var(--radius-md);background:var(--color-error-bg,#fee2e2);color:var(--color-error,#b91c1c);border:1px solid var(--color-error,#b91c1c)">{}</div>"#, h(msg)
        ),
        None => String::new(),
    };

    let content = format!(r#"
<div class="page-header">
  <div>
    <h1 class="page-title">Clone Repository</h1>
    <p class="page-subtitle">Clone a remote repository using an account's SSH key</p>
  </div>
</div>
<div class="page-body">
  <div class="card card--elevated" style="max-width:var(--content-narrow)">
    <div class="card-body">
      {error_banner}
      <form method="POST" action="/clone" novalidate>
        <div class="form-group">
          <label class="form-label" for="full_name">Repository URL or owner/repo</label>
          <input id="full_name" name="full_name" class="input"
                 placeholder="git@github.com:owner/repo.git  or  owner/repo" required>
          <p class="form-hint">SSH URL, HTTPS URL, or shorthand <code>owner/repo</code></p>
        </div>
        <div class="form-group">
          <label class="form-label" for="account">Account</label>
          <select id="account" name="account" class="input" required>
            <option value="" disabled selected>— select account —</option>
            {options}
          </select>
          {no_accounts_hint}
        </div>
        <div class="form-group">
          <label class="form-label" for="dest">Destination Path
            <span class="text-muted font-medium" style="font-size:var(--text-xs);margin-left:4px">(optional)</span>
          </label>
          <input id="dest" name="dest" class="input"
                 placeholder="~/projects/my-repo — defaults to repo name">
        </div>
        <div class="flex justify-end gap-3 mt-6">
          <a href="/repositories" class="btn btn--secondary">Cancel</a>
          <button type="submit" class="btn btn--primary">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            Clone Repository
          </button>
        </div>
      </form>
    </div>
  </div>
</div>"#);

    let breadcrumb = r#"<nav class="breadcrumb"><a href="/">Home</a><span class="breadcrumb-sep">/</span><a href="/repositories">Repositories</a><span class="breadcrumb-sep">/</span><span class="breadcrumb-current">Clone</span></nav>"#;
    Html(page(PageProps { title: "Clone Repository", active: "clone", breadcrumb, topbar_right: "", content: &content }))
}

#[derive(Deserialize)]
pub struct CloneForm { pub full_name: String, pub account: String, pub dest: Option<String> }

pub async fn wizard_submit(
    State(state): State<AppState>,
    Form(form): Form<CloneForm>,
) -> Redirect {
    let acc = match state.services.services()
        .get_account_by_alias(&form.account, None).await
    {
        Ok(Some(a)) => a,
        Ok(None) => {
            let raw = format!("Account '{}' not found.", form.account);
            let msg = urlencoding::encode(&raw);
            return Redirect::to(&format!("/clone?error={msg}"));
        }
        Err(e) => {
            let raw = format!("Database error: {e}");
            let msg = urlencoding::encode(&raw);
            return Redirect::to(&format!("/clone?error={msg}"));
        }
    };

    let dest = std::path::PathBuf::from(form.dest.unwrap_or_else(|| {
        form.full_name.split('/').next_back().unwrap_or("repo").to_string()
    }));

    match state.services.services().clone_repository(CloneRepositoryCommand {
        account_uuid: Some(acc.uuid),
        url:          form.full_name,
        destination:  Some(dest.to_string_lossy().into_owned()),
        ..Default::default()
    }).await {
        Ok(_)  => Redirect::to("/repositories"),
        Err(e) => {
            let raw = format!("Clone failed: {e}");
            let msg = urlencoding::encode(&raw);
            Redirect::to(&format!("/clone?error={msg}"))
        }
    }
}
