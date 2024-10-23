use std::path::PathBuf;

pub fn get_app_path() -> PathBuf {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| String::from(""));

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

    app_data_path
}
