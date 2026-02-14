use serde_json::Value;
use tempfile::TempDir;

/// Initializes a temporary data directory for testing purposes.
///
/// Creates a temp config file pointing to a temp data directory,
/// returns the path to the config file.
pub fn init_test_data_dir() -> String {
    let tmp_dir = TempDir::new().unwrap();
    let tmp_path = tmp_dir.into_path();
    let tmp_dir_path = tmp_path.to_str().unwrap().to_string();

    let data_dir_path = format!("{}/data", tmp_dir_path);
    std::fs::create_dir(&data_dir_path).unwrap();

    let config_path = "config.toml";
    let tmp_config_file_path = format!("{}/config.toml", tmp_dir_path);

    let config_file = std::fs::read_to_string(config_path).unwrap();
    let config_file_toml = toml::from_str::<Value>(&config_file).unwrap();

    let mut updated_config = config_file_toml;
    updated_config["data_dir"] = Value::String(data_dir_path);

    let updated_toml = toml::to_string(&updated_config).unwrap();
    std::fs::write(&tmp_config_file_path, updated_toml).unwrap();

    tmp_config_file_path
}
