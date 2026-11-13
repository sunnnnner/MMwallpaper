mod models;
mod display;
mod wallpaper;
mod style;
mod components;

use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
use std::collections::HashMap;

use models::{AppConfig, MonitorInfo, ScreenWallpaperConfig};
use components::{DisplayCanvas, WallpaperSheet, PreviewModal};

fn main() {
    let window = WindowBuilder::new()
        .with_title("Ubuntu 多屏壁纸管家")
        .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(1080.0, 780.0))
        .with_min_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(860.0, 600.0))
        .with_focused(true)
        .with_resizable(true);

    let cfg = Config::new()
        .with_window(window);

    LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

#[component]
fn App() -> Element {
    let desktop = dioxus::desktop::use_window();
    let mut is_dark = use_signal(|| true);
    let mut monitors = use_signal(|| Vec::<MonitorInfo>::new());
    let mut configs = use_signal(|| Vec::<ScreenWallpaperConfig>::new());
    let mut selected_id = use_signal(|| String::new());
    let mut system_wallpapers = use_signal(|| Vec::<String>::new());
    let mut thumbnails = use_signal(|| HashMap::<String, String>::new());
    let mut applying = use_signal(|| false);
    let mut preview_modal_open = use_signal(|| false);
    let mut preview_b64 = use_signal(|| Option::<String>::None);
    let mut output_path = use_signal(|| Option::<String>::None);
    let mut notification = use_signal(|| Option::<String>::None);

    // 初始化硬件显示屏检测与持久化加载
    use_future(move || async move {
        // 1. 探测显示屏
        if let Ok(mons) = display::detect_monitors() {
            if let Some(prim) = mons.iter().find(|m| m.is_primary).or_else(|| mons.first()) {
                selected_id.set(prim.id.clone());
            }

            // 2. 加载配置
            let saved = wallpaper::load_app_config();
            is_dark.set(saved.dark_mode);

            let merged: Vec<ScreenWallpaperConfig> = mons
                .iter()
                .map(|m| {
                    saved
                        .screens
                        .iter()
                        .find(|s| s.monitor_id == m.id)
                        .cloned()
                        .unwrap_or_else(|| ScreenWallpaperConfig {
                            monitor_id: m.id.clone(),
                            wallpaper_path: None,
                            fit_mode: "cover".to_string(),
                            bg_color: None,
                        })
                })
                .collect();

            // 3. 预先加载已配置壁纸的缩略图
            for c in &merged {
                if let Some(ref p) = c.wallpaper_path {
                    if let Ok(b64) = wallpaper::get_image_thumbnail_base64(p, 360) {
                        thumbnails.write().insert(p.clone(), b64);
                    }
                }
            }

            configs.set(merged);
            monitors.set(mons);
        }

        // 4. 扫描系统壁纸
        let sys_walls = wallpaper::scan_system_wallpapers();
        system_wallpapers.set(sys_walls.clone());

        // 5. 后台全量平滑加载所有壁纸缩略图（优先首批，随后流式全部加载）
        let initial_paths: Vec<String> = sys_walls.iter().take(8).cloned().collect();
        let remaining_paths: Vec<String> = sys_walls.into_iter().skip(8).collect();

        // 优先加载前 8 张保证首屏立即可见
        let mut first_batch = HashMap::new();
        for path in &initial_paths {
            if let Ok(b64) = wallpaper::get_image_thumbnail_base64(path, 300) {
                first_batch.insert(path.clone(), b64);
            }
        }
        thumbnails.write().extend(first_batch);

        // 后台异步任务：流式分批加载剩余所有壁纸，彻底解决“部分加载中”问题
        spawn(async move {
            let mut chunk = HashMap::new();
            let total = remaining_paths.len();
            for (idx, path) in remaining_paths.into_iter().enumerate() {
                if let Ok(b64) = wallpaper::get_image_thumbnail_base64(&path, 300) {
                    chunk.insert(path, b64);
                }
                if chunk.len() >= 4 || idx + 1 == total {
                    thumbnails.write().extend(chunk.drain());
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }
        });
    });

    // 辅助函数：确保缩略图存在
    let mut ensure_thumb = move |path: &str| {
        if !thumbnails.read().contains_key(path) {
            if let Ok(b64) = wallpaper::get_image_thumbnail_base64(path, 320) {
                thumbnails.write().insert(path.to_string(), b64);
            }
        }
    };

    // 切换屏幕
    let handle_select_monitor = move |id: String| {
        selected_id.set(id);
    };

    // 选择本地文件
    let handle_pick_local = move |_| {
        if let Some(file) = rfd::FileDialog::new()
            .set_title("选择壁纸图片")
            .add_filter("图片文件", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        {
            let path_str = file.to_string_lossy().to_string();
            ensure_thumb(&path_str);
            let cur_id = selected_id.read().clone();
            configs.write().iter_mut().for_each(|c| {
                if c.monitor_id == cur_id {
                    c.wallpaper_path = Some(path_str.clone());
                }
            });
            notification.set(Some("已选择本地壁纸图片".to_string()));
        }
    };

    // 切换适应模式
    let handle_change_fit = move |mode: String| {
        let cur_id = selected_id.read().clone();
        configs.write().iter_mut().for_each(|c| {
            if c.monitor_id == cur_id {
                c.fit_mode = mode.clone();
            }
        });
    };

    // 从图库选择壁纸
    let handle_select_wallpaper = move |path: String| {
        ensure_thumb(&path);
        let cur_id = selected_id.read().clone();
        configs.write().iter_mut().for_each(|c| {
            if c.monitor_id == cur_id {
                c.wallpaper_path = Some(path.clone());
            }
        });
    };

    // 清除壁纸
    let handle_clear = move |_| {
        let cur_id = selected_id.read().clone();
        configs.write().iter_mut().for_each(|c| {
            if c.monitor_id == cur_id {
                c.wallpaper_path = None;
            }
        });
        notification.set(Some("已清除当前屏幕壁纸".to_string()));
    };

    // 同步到所有屏幕
    let handle_apply_to_all = move |_| {
        let cur_id = selected_id.read().clone();
        let cur_conf = configs.read().iter().find(|c| c.monitor_id == cur_id).cloned();
        if let Some(source) = cur_conf {
            if let Some(ref path) = source.wallpaper_path {
                configs.write().iter_mut().for_each(|c| {
                    c.wallpaper_path = Some(path.clone());
                    c.fit_mode = source.fit_mode.clone();
                });
                notification.set(Some("已同步壁纸至所有显示器".to_string()));
            }
        }
    };

    // 应用壁纸到 GNOME
    let handle_apply_wallpapers = move |_| {
        let current_mons = monitors.read().clone();
        let current_confs = configs.read().clone();

        let has_any = current_confs.iter().any(|c| c.wallpaper_path.is_some());
        if !has_any {
            notification.set(Some("请至少为一个显示屏配置壁纸图片".to_string()));
            return;
        }

        applying.set(true);
        let dark_mode_val = *is_dark.read();

        // 执行跨屏合成与系统壁纸设置
        match wallpaper::compose_and_apply(&current_mons, &current_confs) {
            Ok(res) => {
                preview_b64.set(res.preview_base64);
                output_path.set(Some(res.output_path));
                notification.set(Some(res.message));

                // 持久化保存
                let _ = wallpaper::save_app_config(&AppConfig {
                    screens: current_confs,
                    dark_mode: dark_mode_val,
                });
            }
            Err(e) => {
                notification.set(Some(format!("应用壁纸失败: {}", e)));
            }
        }
        applying.set(false);
    };

    let active_monitor = monitors.read().iter().find(|m| m.id == *selected_id.read()).cloned();
    let active_config = configs.read().iter().find(|c| c.monitor_id == *selected_id.read()).cloned();
    let current_is_dark = *is_dark.read();

    rsx! {
        style { "{style::build_full_css(current_is_dark)}" }

        div {
            style: "min-height: 100vh; display: flex; flex-direction: column;",

            // 顶部 M3 Top App Bar
            header {
                class: "app-header",
                style: "height: 64px; display: flex; align-items: center; justify-content: space-between; padding: 0 28px; position: sticky; top: 0; z-index: 50;",
                div {
                    style: "display: flex; align-items: center; gap: 12px;",
                    div {
                        style: "width: 36px; height: 36px; border-radius: 12px; background: #0B57D0; color: #FFF; display: flex; align-items: center; justify-content: center; font-size: 18px; box-shadow: 0 2px 8px rgba(11, 87, 208, 0.35);",
                        "🪟"
                    }
                    div {
                        h1 { style: "margin: 0; font-size: 16px; font-weight: 700; letter-spacing: -0.3px;", "Ubuntu 多屏壁纸管家" }
                    }
                }

                div {
                    style: "display: flex; align-items: center; gap: 12px;",

                    // 全景预览按钮
                    if preview_b64.read().is_some() {
                        button {
                            class: "m3-btn m3-btn-tonal",
                            onclick: move |_| preview_modal_open.set(true),
                            "👁️ 全景预览"
                        }
                    }

                    // 深浅色模式切换
                    button {
                        class: "m3-btn m3-btn-tonal",
                        style: "padding: 8px 12px;",
                        onclick: move |_| {
                            let next = !*is_dark.read();
                            is_dark.set(next);
                        },
                        if current_is_dark { "☀️ 明亮" } else { "🌙 暗色" }
                    }

                    // 一键应用按钮
                    button {
                        class: "m3-btn m3-btn-primary",
                        style: "font-size: 13.5px; padding: 8px 22px;",
                        disabled: *applying.read(),
                        onclick: handle_apply_wallpapers,
                        if *applying.read() { "正在合成中..." } else { "✨ 应用多屏壁纸" }
                    }

                    // 退出/关闭窗口
                    button {
                        class: "m3-btn m3-btn-tonal",
                        style: "padding: 8px 12px; opacity: 0.8;",
                        title: "关闭退出应用",
                        onclick: move |_| desktop.close(),
                        "✕"
                    }
                }
            }

            // 浮动提示通知条 (Toast)
            if let Some(ref msg) = *notification.read() {
                div {
                    style: "margin: 12px 28px 0 28px; padding: 10px 18px; border-radius: 9999px; background: rgba(14, 165, 233, 0.2); color: #0EA5E9; border: 1px solid rgba(14, 165, 233, 0.4); font-size: 13px; font-weight: 500; display: flex; justify-content: space-between; align-items: center;",
                    span { "{msg}" }
                    button {
                        style: "background: none; border: none; cursor: pointer; color: inherit; font-size: 14px; font-weight: bold;",
                        onclick: move |_| notification.set(None),
                        "×"
                    }
                }
            }

            // 主体区域
            main {
                style: "flex: 1; max-width: 1380px; width: 100%; margin: 0 auto; padding: 20px 24px 32px 24px; display: flex; flex-direction: column; gap: 18px;",

                // 1. 屏幕空间拓扑画布
                DisplayCanvas {
                    monitors: monitors.read().clone(),
                    configs: configs.read().clone(),
                    selected_id: selected_id.read().clone(),
                    thumbnails: thumbnails.read().clone(),
                    is_dark: current_is_dark,
                    on_select: handle_select_monitor,
                }

                // 2. 当前选中显示器壁纸配置面板与系统壁纸库
                WallpaperSheet {
                    monitor: active_monitor,
                    config: active_config,
                    thumbnails: thumbnails.read().clone(),
                    system_wallpapers: system_wallpapers.read().clone(),
                    is_dark: current_is_dark,
                    on_pick_local: handle_pick_local,
                    on_change_fit: handle_change_fit,
                    on_select_wallpaper: handle_select_wallpaper,
                    on_clear: handle_clear,
                    on_apply_to_all: handle_apply_to_all,
                }
            }

            // 全景跨屏大图预览弹窗
            PreviewModal {
                open: *preview_modal_open.read(),
                preview_base64: preview_b64.read().clone(),
                output_path: output_path.read().clone(),
                monitors: monitors.read().clone(),
                is_dark: current_is_dark,
                on_close: move |_| preview_modal_open.set(false),
            }
        }
    }
}
