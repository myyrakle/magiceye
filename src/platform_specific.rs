use std::path::PathBuf;

use crate::config::Config;

pub fn get_app_path() -> anyhow::Result<PathBuf> {
    // ------- Windows Only
    #[cfg(target_os = "windows")]
    let app_data_path = PathBuf::from(r"\AppData\Local\magiceye");
    // Windows Only -------

    #[cfg(not(any(target_os = "windows")))]
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from(""));

    #[cfg(target_os = "macos")]
    let app_data_path = PathBuf::from(format!("{}/Library/Application Support", home_dir));

    #[cfg(target_os = "linux")]
    let app_data_path = PathBuf::from(format!("{}/.local/share", home_dir));

    // Ensure the directory exists
    if !app_data_path.exists() {
        if let Err(error) = std::fs::create_dir_all(&app_data_path) {
            return Err(anyhow::Error::new(error));
        }
    }

    log::debug!("app_data_path: {app_data_path:?}");

    Ok(app_data_path)
}

pub fn get_config_path() -> anyhow::Result<PathBuf> {
    let app_data_path = get_app_path()?;

    let config_path = app_data_path.join("config.json");

    // Ensure the file exists
    if !config_path.exists() {
        let config = Config::default();
        let config_json = match serde_json::to_string_pretty(&config) {
            Ok(config_json) => config_json,
            Err(error) => {
                return Err(anyhow::Error::new(error));
            }
        };

        if let Err(error) = std::fs::write(&config_path, config_json) {
            return Err(anyhow::Error::new(error));
        }
    }

    log::debug!("config_path: {config_path:?}");

    Ok(config_path)
}

pub fn get_config() -> anyhow::Result<Config> {
    let config_path = get_config_path()?;

    let config_json = match std::fs::read_to_string(&config_path) {
        Ok(config_json) => config_json,
        Err(error) => {
            return Err(anyhow::Error::new(error));
        }
    };

    let config: Config = match serde_json::from_str(&config_json) {
        Ok(config) => config,
        Err(error) => {
            return Err(anyhow::Error::new(error));
        }
    };

    log::debug!("config: {:?}", config);

    Ok(config)
}

pub fn save_config(config: &Config) -> anyhow::Result<()> {
    let config_path = get_config_path()?;

    let config_json = match serde_json::to_string_pretty(config) {
        Ok(config_json) => config_json,
        Err(error) => {
            return Err(anyhow::anyhow!(
                "Failed to serialize config, error: {error}"
            ));
        }
    };

    if let Err(error) = std::fs::write(&config_path, config_json) {
        return Err(anyhow::anyhow!("Failed to save config, error: {error}"));
    }

    Ok(())
}
