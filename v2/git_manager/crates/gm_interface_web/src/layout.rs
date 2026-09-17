use askama_escape::{escape, Html};

/// HTML-escape a user-controlled string for safe interpolation in `format!()` HTML.
pub fn h(s: &str) -> String { escape(s, Html).to_string() }

// Shared page layout — mirrors base.html sidebar structure using themes.css.
// Every page handler calls `page()` to get a consistent shell.

pub struct PageProps<'a> {
    pub title:      &'a str,
    pub active:     &'a str,   // "dashboard" | "accounts" | "repositories" | "ssh" | "clone"
    pub breadcrumb: &'a str,   // inner HTML for the topbar breadcrumb
    pub topbar_right: &'a str, // optional topbar right-side HTML (buttons, etc.)
    pub content:    &'a str,   // full page body HTML
}

/// Renders the complete HTML page using the themes.css design system and
/// the sidebar layout from base.html.
pub fn page(p: PageProps<'_>) -> String {
    let nav_item = |id: &str, href: &str, icon: &str, label: &str| -> String {
        let active_cls = if p.active == id { " active" } else { "" };
        let aria = if p.active == id { r#" aria-current="page""# } else { "" };
        format!(
            r#"<a href="{href}" class="nav-item{active_cls}"{aria}>
        {icon}
        {label}
      </a>"#
        )
    };

    let dashboard_icon = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/></svg>"#;
    let accounts_icon  = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>"#;
    let repos_icon     = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M3 3h18M3 9h18M3 15h18M3 21h18"/></svg>"#;
    let ssh_icon       = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>"#;
    let clone_icon     = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>"#;
    let logo_icon      = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/></svg>"#;

    let n_dashboard   = nav_item("dashboard",    "/",              dashboard_icon, "Dashboard");
    let n_accounts    = nav_item("accounts",     "/accounts",      accounts_icon,  "Accounts");
    let n_repos       = nav_item("repositories", "/repositories",  repos_icon,     "Repositories");
    let n_ssh         = nav_item("ssh",          "/ssh",           ssh_icon,       "SSH Keys");
    let n_clone       = nav_item("clone",        "/clone",         clone_icon,     "Clone Repo");

    format!(r#"<!DOCTYPE html>
<html lang="en" data-theme="">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title} — Git Manager</title>
  <link rel="stylesheet" href="/static/css/themes.css">
</head>
<body>
<div class="app-shell">

  <nav class="sidebar" aria-label="Main navigation">
    <div class="sidebar-logo" aria-label="Git Manager home">
      {logo_icon}
      Git Manager
    </div>
    <div class="sidebar-nav">
      <p class="sidebar-section-title">Workspace</p>
      {n_dashboard}
      {n_accounts}
      {n_repos}
      {n_ssh}
      {n_clone}
    </div>
    <div class="p-4 border-b" style="border-top:1px solid var(--color-border);margin-top:auto;">
      <button class="btn btn--ghost btn--sm" id="theme-toggle" aria-label="Toggle colour theme" title="Toggle dark/light mode" style="width:100%;justify-content:flex-start;gap:8px;">
        <svg id="icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16"><path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/></svg>
        <svg id="icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16" style="display:none"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
        Toggle theme
      </button>
    </div>
  </nav>

  <div class="main-content">
    <header class="topbar" role="banner">
      <div class="flex items-center gap-4">{breadcrumb}</div>
      <div class="flex items-center gap-3">
        {topbar_right}
        <a href="/accounts/new" class="btn btn--primary btn--sm">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><path d="M12 5v14M5 12h14"/></svg>
          Add Account
        </a>
      </div>
    </header>
    <main id="main-content" tabindex="-1">
      {content}
    </main>
  </div>

</div>
<script>
  (function () {{
    var stored = localStorage.getItem('gm-theme') || 'auto';
    var html = document.documentElement;
    if (stored === 'dark')  html.setAttribute('data-theme', 'dark');
    if (stored === 'light') html.setAttribute('data-theme', 'light');
    updateIcon(stored === 'dark' || (stored === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches));
  }})();
  function updateIcon(isDark) {{
    document.getElementById('icon-moon').style.display = isDark ? 'none' : 'block';
    document.getElementById('icon-sun').style.display  = isDark ? 'block' : 'none';
  }}
  document.getElementById('theme-toggle').addEventListener('click', function () {{
    var html = document.documentElement;
    var next = (html.getAttribute('data-theme') === 'dark') ? 'light' : 'dark';
    html.setAttribute('data-theme', next);
    localStorage.setItem('gm-theme', next);
    updateIcon(next === 'dark');
  }});
</script>
</body></html>"#,
        title       = p.title,
        logo_icon   = logo_icon,
        n_dashboard = n_dashboard,
        n_accounts  = n_accounts,
        n_repos     = n_repos,
        n_ssh       = n_ssh,
        n_clone     = n_clone,
        breadcrumb  = p.breadcrumb,
        topbar_right = p.topbar_right,
        content     = p.content,
    )
}
