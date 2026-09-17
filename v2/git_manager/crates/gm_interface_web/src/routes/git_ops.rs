use axum::{extract::State, response::Html};
use crate::state::AppState;

pub async fn ops_page(_: State<AppState>) -> Html<&'static str> {
    Html(r#"<html><body><h1>Git Operations</h1><p>Use the CLI for interactive git operations.</p></body></html>"#)
}
