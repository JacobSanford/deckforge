use tempfile::TempDir;

use crate::config::Config;

/// Initializes a temporary data directory for testing purposes.
///
/// Creates a temp config pointing to a temp data directory.
/// Returns the Config and the TempDir handle (to keep it alive).
pub fn init_test_config() -> (Config, TempDir) {
    let tmp_dir = TempDir::new().unwrap();
    let data_dir_path = format!("{}/data", tmp_dir.path().to_str().unwrap());
    std::fs::create_dir(&data_dir_path).unwrap();

    let config = Config {
        data_dir: data_dir_path,
        listen_addr: None,
        authorized_keys_path: None,
    };

    (config, tmp_dir)
}
