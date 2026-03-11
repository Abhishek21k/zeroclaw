//! Static file serving for the embedded web dashboard.
//!
//! When the `web-dashboard` feature is enabled, this module uses `rust-embed` to bundle the
//! `web/dist/` directory into the binary at compile time. When the feature is disabled, the
//! handlers are still present but return `404 Not Found` so the core gateway can run without
//! pulling the full dashboard assets into the default binary.

use axum::{
    http::{StatusCode, Uri},
    response::IntoResponse,
};

#[cfg(feature = "web-dashboard")]
use axum::http::header;
#[cfg(feature = "web-dashboard")]
use rust_embed::Embed;

// ─────────────────────────────────────────────────────────────────────────────
// Embedded assets implementation (feature = "web-dashboard")
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "web-dashboard")]
#[derive(Embed)]
#[folder = "web/dist/"]
struct WebAssets;

/// Serve static files from `/_app/*` path
#[cfg(feature = "web-dashboard")]
pub async fn handle_static(uri: Uri) -> impl IntoResponse {
    let path = uri.path().strip_prefix("/_app/").unwrap_or(uri.path());

    serve_embedded_file(path)
}

/// SPA fallback: serve index.html for any non-API, non-static GET request
#[cfg(feature = "web-dashboard")]
pub async fn handle_spa_fallback() -> impl IntoResponse {
    serve_embedded_file("index.html")
}

#[cfg(feature = "web-dashboard")]
fn serve_embedded_file(path: &str) -> impl IntoResponse {
    match WebAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, mime),
                    (
                        header::CACHE_CONTROL,
                        if path.contains("assets/") {
                            // Hashed filenames — immutable cache
                            "public, max-age=31536000, immutable".to_string()
                        } else {
                            // index.html etc — no cache
                            "no-cache".to_string()
                        },
                    ),
                ],
                content.data.to_vec(),
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Stub implementation when web dashboard is disabled (default build)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(not(feature = "web-dashboard"))]
pub async fn handle_static(_uri: Uri) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "Web dashboard is not enabled in this build",
    )
        .into_response()
}

#[cfg(not(feature = "web-dashboard"))]
pub async fn handle_spa_fallback() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        "Web dashboard is not enabled in this build",
    )
        .into_response()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(all(test, not(feature = "web-dashboard")))]
mod tests_without_dashboard {
    use super::*;
    use axum::http::Uri;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn static_handler_returns_not_found_when_dashboard_disabled() {
        let uri: Uri = "/_app/index.html".parse().unwrap();
        let response = handle_static(uri).await.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn spa_fallback_returns_not_found_when_dashboard_disabled() {
        let response = handle_spa_fallback().await.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

#[cfg(all(test, feature = "web-dashboard"))]
mod tests_with_dashboard {
    use super::*;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn spa_fallback_serves_index_html_when_dashboard_enabled() {
        let response = handle_spa_fallback().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

