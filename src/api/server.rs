use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::middleware as axum_middleware;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use tokio::sync::RwLock;

use crate::api::middleware::require_auth;
use crate::auth::keys::AuthorizedKeys;
use crate::blockchain::deckchain::DeckChain;
use crate::config::Config;

pub struct AppState {
    pub deckchain: RwLock<DeckChain>,
    pub config: Config,
    pub authorized_keys: AuthorizedKeys,
}

#[derive(Serialize)]
struct ApiErrorResponse {
    error: String,
}

fn json_error(status: StatusCode, message: &str) -> impl IntoResponse {
    (
        status,
        Json(ApiErrorResponse {
            error: message.to_string(),
        }),
    )
}

async fn health() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_blockchain(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let deckchain = state.deckchain.read().await;
    match serde_json::to_string(&deckchain.blockchain) {
        Ok(json) => {
            tracing::info!("Blockchain request");
            (StatusCode::OK, json).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to serialize blockchain: {}", e);
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to serialize blockchain").into_response()
        }
    }
}

async fn get_series_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let deckchain = state.deckchain.read().await;
    let releases = deckchain.card_series_releases();
    Json(releases).into_response()
}

async fn get_series_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let deckchain = state.deckchain.read().await;
    match deckchain.card_series_release(&id) {
        Ok(release) => Json(release).into_response(),
        Err(e) => json_error(StatusCode::NOT_FOUND, &e.to_string()).into_response(),
    }
}

pub fn build_app(state: Arc<AppState>) -> Router {
    let protected = Router::new()
        .route("/blockchain", get(get_blockchain))
        .route("/series", get(get_series_list))
        .route("/series/{id}", get(get_series_by_id))
        .route_layer(axum_middleware::from_fn_with_state(state.clone(), require_auth));

    let public = Router::new()
        .route("/health", get(health));

    public.merge(protected).with_state(state)
}

pub async fn start_server(config: Config) -> crate::error::Result<()> {
    let listen_addr = config.listen_addr().to_string();
    let deckchain = DeckChain::new(&config)?;
    let authorized_keys = AuthorizedKeys::load_from_file(config.authorized_keys_path())
        .unwrap_or_else(|_| {
            tracing::warn!("No authorized_keys file found, starting with empty key set");
            AuthorizedKeys::new()
        });

    let state = Arc::new(AppState {
        deckchain: RwLock::new(deckchain),
        config,
        authorized_keys,
    });

    let app = build_app(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!("Listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use chrono::{Duration, Utc};
    use tempfile::TempDir;
    use tokio::task;

    fn init_test_state() -> (Arc<AppState>, TempDir) {
        let tmp_dir = TempDir::new().unwrap();
        let data_dir_path = format!("{}/data", tmp_dir.path().to_str().unwrap());
        std::fs::create_dir(&data_dir_path).unwrap();

        let config = Config {
            data_dir: data_dir_path,
            listen_addr: Some("127.0.0.1:0".to_string()),
            authorized_keys_path: None,
        };

        let deckchain = DeckChain::new(&config).unwrap();

        let mut authorized_keys = AuthorizedKeys::new();
        authorized_keys.add_key(
            "test".to_string(),
            "test-api-key".to_string(),
            Utc::now() + Duration::hours(1),
        );

        let state = Arc::new(AppState {
            deckchain: RwLock::new(deckchain),
            config,
            authorized_keys,
        });

        (state, tmp_dir)
    }

    async fn spawn_test_server() -> String {
        let (state, _tmp_dir) = init_test_state();
        let app = build_app(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", local_addr);

        task::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        std::mem::forget(_tmp_dir);

        wait_for_server_up(&base_url).await;
        base_url
    }

    async fn wait_for_server_up(base_url: &str) {
        let client = reqwest::Client::new();
        loop {
            if client
                .get(format!("{}/health", base_url))
                .send()
                .await
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }

    async fn send_test_get_request(base_url: &str, endpoint: &str) -> (String, u16) {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}{}", base_url, endpoint))
            .header("X-API-Key", "test-api-key")
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap();
        (body, status)
    }

    #[tokio::test]
    async fn test_health() {
        let base_url = spawn_test_server().await;
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/health", base_url))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_get_blockchain() {
        let base_url = spawn_test_server().await;
        let (body, status) = send_test_get_request(&base_url, "/blockchain").await;
        assert_eq!(status, 200);
        assert!(body.contains("Init"));
    }

    #[tokio::test]
    async fn test_get_blockchain_unauthorized() {
        let base_url = spawn_test_server().await;
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/blockchain", base_url))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 401);
    }

    #[tokio::test]
    async fn test_get_blockchain_forbidden() {
        let base_url = spawn_test_server().await;
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("{}/blockchain", base_url))
            .header("X-API-Key", "wrong-key")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 403);
    }

    #[tokio::test]
    async fn test_get_nonexistent_route() {
        let base_url = spawn_test_server().await;
        let (_body, status) = send_test_get_request(&base_url, "/nonexistent").await;
        assert_eq!(status, 404);
    }

    #[tokio::test]
    async fn test_get_blockchain_method_not_allowed() {
        let base_url = spawn_test_server().await;
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{}/blockchain", base_url))
            .header("X-API-Key", "test-api-key")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 405);
    }

    #[tokio::test]
    async fn test_get_series_list() {
        let base_url = spawn_test_server().await;
        let (body, status) = send_test_get_request(&base_url, "/series").await;
        assert_eq!(status, 200);
        assert_eq!(body, "[]");
    }

    #[tokio::test]
    async fn test_get_series_not_found() {
        let base_url = spawn_test_server().await;
        let (body, status) = send_test_get_request(&base_url, "/series/nonexistent").await;
        assert_eq!(status, 404);
        assert_eq!(status, 404, "body was: {}", body);
    }
}
