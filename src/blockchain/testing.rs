use tempdir::TempDir;
use serde_json::Value;

// This function initializes a temporary data directory for testing purposes.
//
// It creates a temporary blockchain location.
pub fn init_test_data_dir() -> String {
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

    tmp_config_dir_path
}
