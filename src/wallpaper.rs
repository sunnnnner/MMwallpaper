use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;
use image::imageops::{crop_imm, overlay, resize, FilterType};
use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
use crate::models::{AppConfig, ApplyResult, MonitorInfo, ScreenWallpaperConfig};

fn get_config_file_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("ubuntu-wall")
        .join("config.json")
}

pub fn load_app_config() -> AppConfig {
    let path = get_config_file_path();
    if !path.exists() {
        return AppConfig::default();
    }
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(conf) = serde_json::from_str(&content) {
            return conf;
        }
    }
    AppConfig::default()
}

pub fn save_app_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_file_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json_str = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&path, json_str)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    Ok(())
}

fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut chunks = data.chunks_exact(3);
    for chunk in &mut chunks {
        let b = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
        out.push(TABLE[((b >> 18) & 63) as usize] as char);
        out.push(TABLE[((b >> 12) & 63) as usize] as char);
        out.push(TABLE[((b >> 6) & 63) as usize] as char);
        out.push(TABLE[(b & 63) as usize] as char);
    }
    let rem = chunks.remainder();
    if rem.len() == 1 {
        let b = (rem[0] as u32) << 16;
        out.push(TABLE[((b >> 18) & 63) as usize] as char);
        out.push(TABLE[((b >> 12) & 63) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem.len() == 2 {
        let b = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
        out.push(TABLE[((b >> 18) & 63) as usize] as char);
        out.push(TABLE[((b >> 12) & 63) as usize] as char);
        out.push(TABLE[((b >> 6) & 63) as usize] as char);
        out.push('=');
    }
    out
}

pub fn get_image_thumbnail_base64(path: &str, max_edge: u32) -> Result<String, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(format!("文件不存在: {}", path));
    }

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("ubuntu-wall")
        .join("thumbnails");
    let _ = fs::create_dir_all(&cache_dir);

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&path, &mut hasher);
    let hash_key = std::hash::Hasher::finish(&hasher);
    let cache_file = cache_dir.join(format!("{:016x}_{}.jpg", hash_key, max_edge));

    if cache_file.exists() {
        if let Ok(meta_orig) = p.metadata() {
            if let Ok(meta_cache) = cache_file.metadata() {
                if let (Ok(time_orig), Ok(time_cache)) = (meta_orig.modified(), meta_cache.modified()) {
                    if time_cache >= time_orig {
                        if let Ok(bytes) = fs::read(&cache_file) {
                            let b64 = base64_encode(&bytes);
                            return Ok(format!("data:image/jpeg;base64,{}", b64));
                        }
                    }
                }
            }
        }
    }

    let img = image::open(p).map_err(|e| format!("无法打开图片 {}: {}", path, e))?;
    let (w, h) = img.dimensions();
    let scale = (max_edge as f64 / w.max(h) as f64).min(1.0);
    let tw = (w as f64 * scale).max(1.0).round() as u32;
    let th = (h as f64 * scale).max(1.0).round() as u32;

    let thumb = img.resize(tw, th, FilterType::Triangle);
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    thumb.write_to(&mut cursor, ImageFormat::Jpeg)
        .map_err(|e| format!("缩略图编码失败: {}", e))?;

    let _ = fs::write(&cache_file, &buf);

    let b64 = base64_encode(&buf);
    Ok(format!("data:image/jpeg;base64,{}", b64))
}

pub fn scan_system_wallpapers() -> Vec<String> {
    let dirs_to_check = vec![
        PathBuf::from("/usr/share/backgrounds"),
        dirs::picture_dir().unwrap_or_default(),
    ];

    let mut results = Vec::new();
    for base in dirs_to_check {
        if !base.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&base) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                        let lower = ext.to_lowercase();
                        if matches!(lower.as_str(), "jpg" | "jpeg" | "png" | "webp") {
                            results.push(p.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    results.sort();
    results
}

fn render_screen_wallpaper(
    img: &DynamicImage,
    target_w: u32,
    target_h: u32,
    fit_mode: &str,
) -> RgbaImage {
    let (orig_w, orig_h) = img.dimensions();
    if orig_w == 0 || orig_h == 0 {
        return RgbaImage::from_pixel(target_w, target_h, Rgba([18, 18, 18, 255]));
    }

    match fit_mode {
        "contain" => {
            let scale_x = target_w as f64 / orig_w as f64;
            let scale_y = target_h as f64 / orig_h as f64;
            let scale = scale_x.min(scale_y);
            let new_w = (orig_w as f64 * scale).max(1.0).round() as u32;
            let new_h = (orig_h as f64 * scale).max(1.0).round() as u32;
            let resized = resize(img, new_w, new_h, FilterType::Lanczos3);

            let mut container = RgbaImage::from_pixel(target_w, target_h, Rgba([18, 18, 18, 255]));
            let off_x = (target_w.saturating_sub(new_w)) / 2;
            let off_y = (target_h.saturating_sub(new_h)) / 2;
            overlay(&mut container, &resized, off_x as i64, off_y as i64);
            container
        }
        "stretch" => resize(img, target_w, target_h, FilterType::Lanczos3),
        _ => {
            let scale_x = target_w as f64 / orig_w as f64;
            let scale_y = target_h as f64 / orig_h as f64;
            let scale = scale_x.max(scale_y);
            let new_w = (orig_w as f64 * scale).max(1.0).round() as u32;
            let new_h = (orig_h as f64 * scale).max(1.0).round() as u32;
            let resized = resize(img, new_w, new_h, FilterType::Lanczos3);

            let crop_x = (new_w.saturating_sub(target_w)) / 2;
            let crop_y = (new_h.saturating_sub(target_h)) / 2;
            crop_imm(&resized, crop_x, crop_y, target_w, target_h).to_image()
        }
    }
}

pub fn compose_and_apply(
    monitors: &[MonitorInfo],
    configs: &[ScreenWallpaperConfig],
) -> Result<ApplyResult, String> {
    if monitors.is_empty() {
        return Err("当前无可用显示器".to_string());
    }

    let min_x = monitors.iter().map(|m| m.x).min().unwrap_or(0);
    let min_y = monitors.iter().map(|m| m.y).min().unwrap_or(0);
    let max_x = monitors.iter().map(|m| m.x + m.width as i32).max().unwrap_or(1920);
    let max_y = monitors.iter().map(|m| m.y + m.height as i32).max().unwrap_or(1080);

    let canvas_w = (max_x - min_x).max(100) as u32;
    let canvas_h = (max_y - min_y).max(100) as u32;

    let mut canvas = RgbaImage::from_pixel(canvas_w, canvas_h, Rgba([24, 24, 27, 255]));

    for monitor in monitors {
        let conf = configs.iter().find(|c| c.monitor_id == monitor.id);
        let wallpaper_path = conf.and_then(|c| c.wallpaper_path.as_ref());
        let fit_mode = conf.map(|c| c.fit_mode.as_str()).unwrap_or("cover");

        let rendered_part = if let Some(path) = wallpaper_path {
            if Path::new(path).exists() {
                match image::open(path) {
                    Ok(img) => render_screen_wallpaper(&img, monitor.width, monitor.height, fit_mode),
                    Err(_) => RgbaImage::from_pixel(monitor.width, monitor.height, Rgba([30, 30, 36, 255])),
                }
            } else {
                RgbaImage::from_pixel(monitor.width, monitor.height, Rgba([30, 30, 36, 255]))
            }
        } else {
            RgbaImage::from_pixel(monitor.width, monitor.height, Rgba([30, 30, 36, 255]))
        };

        let target_x = (monitor.x - min_x) as i64;
        let target_y = (monitor.y - min_y) as i64;
        overlay(&mut canvas, &rendered_part, target_x, target_y);
    }

    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("ubuntu-wall");
    fs::create_dir_all(&data_dir)
        .map_err(|e| format!("无法创建存储目录 {:?}: {}", data_dir, e))?;

    let out_path = data_dir.join("ubuntu_spanned_wallpaper.png");
    canvas
        .save(&out_path)
        .map_err(|e| format!("保存跨屏壁纸失败: {}", e))?;

    let uri = format!("file://{}", out_path.display());
    
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-options", "spanned"])
        .output();

    let res1 = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri", &uri])
        .output();

    let res2 = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.background", "picture-uri-dark", &uri])
        .output();

    if res1.is_err() && res2.is_err() {
        return Err("调用 gsettings 设置 GNOME 壁纸失败".to_string());
    }

    let preview_scale = (960.0 / canvas_w as f64).min(1.0);
    let pw = (canvas_w as f64 * preview_scale).round() as u32;
    let ph = (canvas_h as f64 * preview_scale).round() as u32;
    let preview_img = resize(&canvas, pw, ph, FilterType::Triangle);
    
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    let preview_b64 = if preview_img.write_to(&mut cursor, ImageFormat::Jpeg).is_ok() {
        Some(format!("data:image/jpeg;base64,{}", base64_encode(&buf)))
    } else {
        None
    };

    Ok(ApplyResult {
        success: true,
        message: format!("已成功为 {} 个屏幕合成并设置独立壁纸！", monitors.len()),
        output_path: out_path.to_string_lossy().to_string(),
        preview_base64: preview_b64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_system_wallpapers() {
        let walls = scan_system_wallpapers();
        assert!(!walls.is_empty());
    }

    #[test]
    fn test_all_thumbnails() {
        let walls = scan_system_wallpapers();
        let t0 = std::time::Instant::now();
        let mut count = 0;
        for w in &walls {
            if let Ok(_) = get_image_thumbnail_base64(w, 240) {
                count += 1;
            }
        }
        println!("Generated/Loaded {}/{} thumbnails in {:?}", count, walls.len(), t0.elapsed());
    }
}
