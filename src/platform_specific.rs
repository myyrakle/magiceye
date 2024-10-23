use std::path::PathBuf;

use crate::config::Config;

pub fn get_app_path() -> PathBuf {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| String::from(""));

    log::debug!("home_dir: {home_dir}");

    #[cfg(target_os = "windows")]
    let app_data_path = PathBuf::from(format!(r"{}\AppData\Local\magiceye", home_dir));

    #[cfg(target_os = "macos")]
    let app_data_path = PathBuf::from(format!("{}/Library/Application Support", home_dir));

    #[cfg(target_os = "linux")]
    let app_data_path = PathBuf::from(format!("{}/.local/share", home_dir));

    // Ensure the directory exists
    if !app_data_path.exists() {
        std::fs::create_dir_all(&app_data_path).expect("Failed to create app data directory");
    }

    log::debug!("app_data_path: {app_data_path:?}");

    app_data_path
}

pub fn get_config_path() -> PathBuf {
    let app_data_path = get_app_path();

    let config_path = app_data_path.join("config.json");

    // Ensure the file exists
    if !config_path.exists() {
        let config = Config::default();
        let config_json =
            serde_json::to_string_pretty(&config).expect("Failed to serialize config");

        std::fs::write(&config_path, config_json).expect("Failed to create config file");
    }

    log::debug!("config_path: {config_path:?}");

    config_path
}

pub fn get_config() -> Config {
    let config_path = get_config_path();

    let config_json = std::fs::read_to_string(&config_path).expect("Failed to read config file");

    let config: Config = serde_json::from_str(&config_json).expect("Failed to parse config");

    log::debug!("config: {:?}", config);

    config
}

pub fn save_config(config: &Config) {
    let config_path = get_config_path();

    let config_json = serde_json::to_string_pretty(config).expect("Failed to serialize config");

    std::fs::write(&config_path, config_json).expect("Failed to save config");
}
