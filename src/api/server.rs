use std::convert::Infallible;
use std::sync::{Mutex, Arc};

use hyper::{Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::Body;
use lazy_static::lazy_static;
use matchit::Router;
use serde_json::Value;

use crate::blockchain::chain::BlockChain;
use crate::logging::server::{log_info, log_error};
use crate::config::Config;

lazy_static! {
    static ref BLOCKCHAIN_MUTEX: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

async fn handle_request(req: Request<Body>, config: Arc<Config>) -> Result<Response<Body>, Infallible> {
    let mut router = Router::new();
    router.insert("/blockchain", "blockchain").unwrap();

    let matched = router.at(req.uri().path());
    let data_dir = config.data_dir.clone();

    match matched {
        Ok(matched) => {
            match *matched.value {
                "blockchain" => {
                    if !allow_get_only(&req) {
                        return Ok(method_not_allowed());
                    }

                    match blockchain(data_dir) {
                        Ok(blockchain) => {
                            let blockchain_json = serde_json::to_string(&blockchain).unwrap();
                            log_info("Blockchain request".to_string());
                            Ok(Response::new(Body::from(blockchain_json)))
                        },
                        Err(_) => {
                            log_error("Failed to get blockchain".to_string());
                            Ok(internal_server_error())
                        },
                    }
                },
                _ => {
                    log_error(format!("Routing matched, but default handler for path fired: {}", req.uri().path()));
                    Ok(not_found())
                },
            }
        },
        Err(_) => {
            log_error(format!("No handler for path: {}", req.uri().path()));
            Ok(not_found())
        },
    }
}

fn allow_get_only(req: &Request<Body>) -> bool {
    if req.method() == hyper::Method::GET {
        true
    } else {
        log_error(format!("{} method not allowed for {}", req.method(), req.uri().path()));
        false
    }
}

fn blockchain(data_dir: String) -> Result<BlockChain, ()> {
    let storage_path = format!("{}/blockchain.json", data_dir);
    let blockchain = BlockChain::new(&storage_path, Value::Null);
    match blockchain {
        Ok(blockchain) => Ok(blockchain),
        Err(_) => {
            log_error("Failed to initialize blockchain".to_string());
            Err(())
        },
    }
}

fn blockchain_lock(data_dir: String) -> Result<BlockChain, ()> {
    let _lock = BLOCKCHAIN_MUTEX.lock().unwrap();
    blockchain(data_dir)
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(404)
        .body(Body::from("Not Found"))
        .unwrap()
}

fn internal_server_error() -> Response<Body> {
    Response::builder()
        .status(500)
        .body(Body::from("Internal Server Error"))
        .unwrap()
}

fn method_not_allowed() -> Response<Body> {
    Response::builder()
        .status(405)
        .body(Body::from("Method Not Allowed"))
        .unwrap()
}

pub async fn start_server(config_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config::load(config_path.as_str())?);

    let make_svc = make_service_fn(move |_| {
        let config = config.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(req, config.clone())
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use hyper::{Client, StatusCode, Method};
    use hyper::body::to_bytes;
    use tempdir::TempDir;
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
        let (_body, status) = send_test_request("/blockchain", Method::POST, Body::empty(), vec![]).await;
        assert_eq!(status, 405);
    }

    pub async fn sleep_for_server_up() {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    pub async fn wait_for_server_up() {
        let uri = format!("http://127.0.0.1:3000{}", "/blockchain");
        let client = Client::new();
        loop {
            let resp = client.get(uri.parse().unwrap()).await;
            if resp.is_ok() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    pub async fn send_test_get_request(endpoint: &str) -> (String, StatusCode) {
        let (body_string, status) = send_test_request(endpoint, Method::GET, Body::empty(), vec![]).await;
        (body_string, status)
    }

    pub async fn send_test_request(endpoint: &str, method: Method, body: Body, headers: Vec<(&str, &str)>) -> (String, StatusCode) {
        let uri = format!("http://127.0.0.1:3000{}", endpoint);
        let mut req_builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json");

        for (key, value) in headers {
            req_builder = req_builder.header(key, value);
        }

        let req = req_builder.body(body);
        let client = Client::new();
        let resp = client.request(req.unwrap()).await.unwrap();
        let status = resp.status();
        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
        (body_string, status)
    }

    pub async fn spawn_test_server() -> String {
        let test_data_dir: String = init_test_data_dir().await;
        let test_data_clone = test_data_dir.clone();
        task::spawn(async {
            start_server(test_data_dir).await.unwrap();
        });
        wait_for_server_up().await;
        test_data_clone
    }

    async fn init_test_data_dir() -> String {
        let tmp_config_dir = TempDir::new("deckforgetmp").unwrap();
        let tmp_path = tmp_config_dir.into_path();
        let tmp_config_dir_path = tmp_path.to_str().unwrap().to_string();
    
        // Make a data directory in the temp dir
        let data_dir_path = format!("{}/data", tmp_config_dir_path);
        std::fs::create_dir(&data_dir_path).unwrap();

        // Update the line containing data_dir in the config file
        let config_path = "config.toml";
        let tmp_config_file_path = format!("{}/config.toml", tmp_config_dir_path);

        // Read the config file as toml.
        let config_file = std::fs::read_to_string(config_path).unwrap();
        let config_file_toml = toml::from_str::<Value>(&config_file).unwrap();

        // Update the data_dir field in the config file.
        let mut updated_config_file = config_file_toml.clone();
        updated_config_file["data_dir"] = Value::String(data_dir_path.clone());

        // Write the updated config file to the temp directory.
        let updated_config_file_toml = toml::to_string(&updated_config_file).unwrap();
        std::fs::write(&tmp_config_file_path, updated_config_file_toml).unwrap();

        tmp_config_file_path
    }

}
