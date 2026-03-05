use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

use crate::api::server::AppState;

#[derive(Serialize)]
struct AuthError {
    error: String,
}

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match api_key {
        None => (
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                error: "Missing X-API-Key header".to_string(),
            }),
        )
            .into_response(),
        Some(key) => {
            if state.authorized_keys.is_key_authorized(key) {
                next.run(request).await
            } else {
                (
                    StatusCode::FORBIDDEN,
                    Json(AuthError {
                        error: "Invalid or expired API key".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    }
}
