use serde::Serialize;

/// Application facts authored by the backend and rendered as-is by the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub app_name: String,
    pub app_version: String,
    pub game: String,
    pub platform: String,
    pub arch: String,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        app_name: "MaddenTool".into(),
        app_version: env!("CARGO_PKG_VERSION").into(),
        game: "Madden 27".into(),
        platform: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
    }
}
