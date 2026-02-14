use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

use crate::blockchain::chain::BlockChain;
use crate::config::Config;

struct AppState {
    config: Config,
}

async fn get_blockchain(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let storage_path = format!("{}/blockchain.json", state.config.data_dir);
    match BlockChain::new(&storage_path, serde_json::Value::Null) {
        Ok(blockchain) => {
            match serde_json::to_string(&blockchain) {
                Ok(json) => {
                    tracing::info!("Blockchain request");
                    (StatusCode::OK, json).into_response()
                }
                Err(e) => {
                    tracing::error!("Failed to serialize blockchain: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to get blockchain: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn start_server(config_path: String) -> anyhow::Result<()> {
    let config = Config::load(config_path.as_str())?;

    let state = Arc::new(AppState { config });

    let app = Router::new()
        .route("/blockchain", get(get_blockchain))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use tempfile::TempDir;
    use tokio::task;

    #[tokio::test]
    async fn test_get_blockchain() {
        spawn_test_server().await;
        let (body, status) = send_test_get_request("/blockchain").await;
        assert_eq!(status, 200);
        assert!(body.contains("Init"));
    }

    #[tokio::test]
    async fn test_get_nonexistent_route() {
        spawn_test_server().await;
        let (_body, status) = send_test_get_request("/nonexistent").await;
        assert_eq!(status, 404);
    }

    #[tokio::test]
    async fn test_get_blockchain_method_not_allowed() {
        spawn_test_server().await;
        let client = reqwest::Client::new();
        let resp = client
            .post("http://127.0.0.1:3000/blockchain")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 405);
    }

    async fn wait_for_server_up() {
        let client = reqwest::Client::new();
        loop {
            if client
                .get("http://127.0.0.1:3000/blockchain")
                .send()
                .await
                .is_ok()
            {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    async fn send_test_get_request(endpoint: &str) -> (String, u16) {
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://127.0.0.1:3000{}", endpoint))
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap();
        (body, status)
    }

    async fn spawn_test_server() -> String {
        let test_config_path = init_test_data_dir();
        let config_path_clone = test_config_path.clone();
        task::spawn(async move {
            start_server(config_path_clone).await.unwrap();
        });
        wait_for_server_up().await;
        test_config_path
    }

    fn init_test_data_dir() -> String {
        let tmp_dir = TempDir::new().unwrap();
        let tmp_path = tmp_dir.into_path();
        let tmp_dir_path = tmp_path.to_str().unwrap().to_string();

        let data_dir_path = format!("{}/data", tmp_dir_path);
        std::fs::create_dir(&data_dir_path).unwrap();

        let config_path = "config.toml";
        let tmp_config_file_path = format!("{}/config.toml", tmp_dir_path);

        let config_file = std::fs::read_to_string(config_path).unwrap();
        let config_toml: serde_json::Value = toml::from_str(&config_file).unwrap();

        let mut updated = config_toml;
        updated["data_dir"] = serde_json::Value::String(data_dir_path);

        let updated_toml = toml::to_string(&updated).unwrap();
        std::fs::write(&tmp_config_file_path, updated_toml).unwrap();

        tmp_config_file_path
    }
}
