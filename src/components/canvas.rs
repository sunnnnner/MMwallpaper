use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::{MonitorInfo, ScreenWallpaperConfig};

#[component]
pub fn DisplayCanvas(
    monitors: Vec<MonitorInfo>,
    configs: Vec<ScreenWallpaperConfig>,
    selected_id: String,
    thumbnails: HashMap<String, String>,
    is_dark: bool,
    on_select: EventHandler<String>,
) -> Element {
    if monitors.is_empty() {
        return rsx! {
            div {
                class: "monitor-canvas-bg",
                style: "height: 240px; border-radius: 24px; display: flex; align-items: center; justify-content: center; flex-direction: column; gap: 8px;",
                span { style: "font-size: 14px; opacity: 0.6;", "正在探测系统显示设备..." }
            }
        };
    }

    let min_x = monitors.iter().map(|m| m.x).min().unwrap_or(0);
    let min_y = monitors.iter().map(|m| m.y).min().unwrap_or(0);
    let max_x = monitors.iter().map(|m| m.x + m.width as i32).max().unwrap_or(3840);
    let max_y = monitors.iter().map(|m| m.y + m.height as i32).max().unwrap_or(2160);

    let total_w = (max_x - min_x).max(1) as f64;
    let total_h = (max_y - min_y).max(1) as f64;

    let max_view_w = 780.0;
    let max_view_h = 220.0;
    let scale = (max_view_w / total_w).min(max_view_h / total_h);

    let canvas_pixel_w = (total_w * scale).round() as i32;
    let canvas_pixel_h = (total_h * scale).round() as i32;

    rsx! {
        div {
            class: "monitor-canvas-bg",
            style: "border-radius: 24px; padding: 20px 24px; display: flex; flex-direction: column; align-items: center; position: relative;",

            // 顶部信息条
            div {
                style: "width: 100%; display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",
                div {
                    style: "display: flex; align-items: center; gap: 10px;",
                    span {
                        style: "font-size: 12px; font-weight: 700; letter-spacing: 1px; text-transform: uppercase; opacity: 0.65;",
                        "显示器物理拓扑"
                    }
                    span {
                        class: "pulse-badge",
                        style: "background: rgba(14, 165, 233, 0.15); color: #0EA5E9; border: 1px solid rgba(14, 165, 233, 0.3);",
                        "已连接 {monitors.len()} 台屏幕"
                    }
                }
                span {
                    style: "font-size: 12px; opacity: 0.55;",
                    "点击显示屏切换独立配置"
                }
            }

            // 比例缩放模拟画布
            div {
                style: "width: {canvas_pixel_w}px; height: {canvas_pixel_h}px; position: relative; margin: 12px 0 20px 0;",
                for monitor in monitors.iter() {
                    {
                        let is_sel = monitor.id == selected_id;
                        let conf = configs.iter().find(|c| c.monitor_id == monitor.id);
                        let thumb = conf
                            .and_then(|c| c.wallpaper_path.as_ref())
                            .and_then(|p| thumbnails.get(p).cloned());
                        let fit_mode = conf.map(|c| c.fit_mode.as_str()).unwrap_or("cover");

                        let left = ((monitor.x - min_x) as f64 * scale).round() as i32;
                        let top = ((monitor.y - min_y) as f64 * scale).round() as i32;
                        let width = (monitor.width as f64 * scale).round() as i32;
                        let height = (monitor.height as f64 * scale).round() as i32;

                        let object_fit = match fit_mode {
                            "contain" => "contain",
                            "stretch" => "fill",
                            _ => "cover",
                        };

                        let border_style = if is_sel {
                            if is_dark {
                                "2.5px solid #A8C7FA; box-shadow: 0 0 0 4px rgba(168, 199, 250, 0.25);"
                            } else {
                                "2.5px solid #0B57D0; box-shadow: 0 0 0 4px rgba(11, 87, 208, 0.2);"
                            }
                        } else {
                            if is_dark {
                                "1.5px solid #3F3F4D;"
                            } else {
                                "1.5px solid #D2D2DC;"
                            }
                        };

                        let monitor_id = monitor.id.clone();

                        rsx! {
                            div {
                                key: "{monitor.id}",
                                onclick: move |_| on_select.call(monitor_id.clone()),
                                style: "position: absolute; left: {left}px; top: {top}px; width: {width}px; height: {height}px; border-radius: 14px; overflow: hidden; cursor: pointer; display: flex; flex-direction: column; justify-content: space-between; transition: all 0.2s cubic-bezier(0.2, 0, 0, 1); {border_style}",

                                // 壁纸背景层
                                if let Some(t) = thumb {
                                    img {
                                        src: "{t}",
                                        style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; object-fit: {object_fit}; background: #0B0B0D; z-index: 1;"
                                    }
                                } else {
                                    div {
                                        style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: rgba(120, 120, 130, 0.15); display: flex; align-items: center; justify-content: center; z-index: 1;",
                                        span { style: "font-size: 11px; opacity: 0.5;", "未设置壁纸" }
                                    }
                                }

                                // 顶部标签
                                div {
                                    style: "position: relative; z-index: 2; padding: 6px 8px; display: flex; justify-content: space-between; align-items: center; background: linear-gradient(180deg, rgba(0,0,0,0.65) 0%, rgba(0,0,0,0) 100%);",
                                    div {
                                        style: "display: flex; align-items: center; gap: 4px;",
                                        if monitor.is_primary {
                                            span {
                                                style: "background: #F59E0B; color: #FFF; font-size: 9px; font-weight: 700; padding: 1px 5px; border-radius: 9999px;",
                                                "★ 主屏"
                                            }
                                        }
                                        span {
                                            style: "color: #FFF; font-size: 11px; font-weight: 600; text-shadow: 0 1px 2px rgba(0,0,0,0.8);",
                                            "{monitor.id}"
                                        }
                                    }
                                    if is_sel {
                                        span {
                                            style: "background: #0B57D0; color: #FFF; font-size: 9px; font-weight: 700; padding: 1px 6px; border-radius: 9999px;",
                                            "编辑中"
                                        }
                                    }
                                }

                                // 底部规格条
                                div {
                                    style: "position: relative; z-index: 2; padding: 5px 8px; background: linear-gradient(0deg, rgba(0,0,0,0.7) 0%, rgba(0,0,0,0) 100%); display: flex; justify-content: space-between; align-items: flex-end;",
                                    span {
                                        style: "color: rgba(255,255,255,0.9); font-size: 10.5px; font-weight: 500;",
                                        "{monitor.width}×{monitor.height}"
                                    }
                                    span {
                                        style: "color: rgba(255,255,255,0.7); font-size: 9.5px; text-transform: uppercase;",
                                        "{fit_mode}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 底部快速选择器
            div {
                style: "display: flex; gap: 10px; flex-wrap: wrap; justify-content: center;",
                for monitor in monitors.iter() {
                    {
                        let is_sel = monitor.id == selected_id;
                        let btn_class = if is_sel { "m3-btn m3-btn-primary" } else { "m3-btn m3-btn-tonal" };
                        let monitor_id = monitor.id.clone();
                        rsx! {
                            button {
                                key: "{monitor.id}",
                                class: "{btn_class}",
                                onclick: move |_| on_select.call(monitor_id.clone()),
                                span { "{monitor.name}" }
                                span { style: "opacity: 0.65; font-size: 11px;", "({monitor.width}×{monitor.height})" }
                            }
                        }
                    }
                }
            }
        }
    }
}
