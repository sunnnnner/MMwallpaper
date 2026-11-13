use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale: f64,
    pub is_primary: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScreenWallpaperConfig {
    pub monitor_id: String,
    pub wallpaper_path: Option<String>,
    pub fit_mode: String, // "cover" | "contain" | "stretch"
    pub bg_color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub screens: Vec<ScreenWallpaperConfig>,
    pub dark_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplyResult {
    pub success: bool,
    pub message: String,
    pub output_path: String,
    pub preview_base64: Option<String>,
}
