// Serves embedded static assets (CSS, JS) from the binary.
// In production these would be served by nginx; in development axum handles them.
use axum::{extract::Path, http::{header, StatusCode}, response::Response};

pub async fn serve_static(Path(path): Path<String>) -> Response {
    let (body, content_type): (&'static str, &'static str) = match path.as_str() {
        "css/main.css"       => (include_str!("../static/css/main.css"),       "text/css"),
        "css/themes.css"     => (include_str!("../static/css/themes.css"),     "text/css"),
        "css/components.css" => (include_str!("../static/css/components.css"), "text/css"),
        "js/main.js"         => (include_str!("../static/js/main.js"),         "application/javascript"),
        "js/api.js"          => (include_str!("../static/js/api.js"),          "application/javascript"),
        "js/sse_client.js"   => (include_str!("../static/js/sse_client.js"),   "application/javascript"),
        _ => return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::empty())
            .expect("static files: 404 Response builder should not fail"),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "max-age=86400")
        .body(axum::body::Body::from(body))
        .expect("static files: Response builder should not fail with valid headers and body")
}
