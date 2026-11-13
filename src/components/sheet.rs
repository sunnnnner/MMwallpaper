use dioxus::prelude::*;
use std::collections::HashMap;
use crate::models::{MonitorInfo, ScreenWallpaperConfig};

#[component]
pub fn WallpaperSheet(
    monitor: Option<MonitorInfo>,
    config: Option<ScreenWallpaperConfig>,
    thumbnails: HashMap<String, String>,
    system_wallpapers: Vec<String>,
    is_dark: bool,
    on_pick_local: EventHandler<()>,
    on_change_fit: EventHandler<String>,
    on_select_wallpaper: EventHandler<String>,
    on_clear: EventHandler<()>,
    on_apply_to_all: EventHandler<()>,
) -> Element {
    let m = match monitor {
        Some(val) => val,
        None => return rsx! { div {} },
    };

    let current_path = config.as_ref().and_then(|c| c.wallpaper_path.clone());
    let current_thumb = current_path.as_ref().and_then(|p| thumbnails.get(p).cloned());
    let current_fit = config.as_ref().map(|c| c.fit_mode.clone()).unwrap_or_else(|| "cover".to_string());

    // 根据当前选中的物理显示器宽高自适应计算卡片比例（标准 W3C aspect-ratio: W / H 语法）
    let (ratio_w, ratio_h) = if m.width > 0 && m.height > 0 {
        (m.width, m.height)
    } else {
        (16, 9)
    };

    let ratio_name = if ratio_w * 9 == ratio_h * 16 {
        "16:9"
    } else if ratio_w * 10 == ratio_h * 16 {
        "16:10"
    } else if ratio_w * 9 == ratio_h * 21 {
        "21:9"
    } else {
        "物理等比"
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 16px; width: 100%;",

            // 1. 活跃屏幕状态控制卡
            div {
                class: "m3-card m3-card-bg",
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 16px;",
                    div {
                        div {
                            style: "display: flex; align-items: center; gap: 8px;",
                            h3 { style: "font-size: 18px; font-weight: 700; margin: 0;", "{m.name}" }
                            if m.is_primary {
                                span {
                                    class: "pulse-badge",
                                    style: "background: #F59E0B; color: #FFF;",
                                    "主显示器"
                                }
                            }
                        }
                        p {
                            style: "margin: 4px 0 0 0; font-size: 13px; opacity: 0.65;",
                            "物理分辨率：{m.width} × {m.height} 像素 · 比例：{ratio_name} · 坐标原点：({m.x}, {m.y})"
                        }
                    }

                    // 操作按钮
                    div {
                        style: "display: flex; gap: 8px; flex-wrap: wrap;",
                        button {
                            class: "m3-btn m3-btn-primary",
                            onclick: move |_| on_pick_local.call(()),
                            "📁 选择本地壁纸"
                        }
                        button {
                            class: "m3-btn m3-btn-tonal",
                            disabled: current_path.is_none(),
                            onclick: move |_| on_apply_to_all.call(()),
                            "⎘ 同步至所有屏幕"
                        }
                        button {
                            class: "m3-btn m3-btn-danger",
                            disabled: current_path.is_none(),
                            onclick: move |_| on_clear.call(()),
                            "🗑 清除壁纸"
                        }
                    }
                }

                // 适应方式与当前壁纸信息
                div {
                    style: "margin-top: 18px; padding-top: 14px; border-top: 1px solid rgba(140, 140, 150, 0.15); display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px;",
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        span { style: "font-size: 13px; font-weight: 500; opacity: 0.8;", "适应模式：" }
                        div {
                            style: "display: inline-flex; background: rgba(140, 140, 150, 0.15); padding: 3px; border-radius: 9999px; gap: 4px;",
                            for (mode_key, mode_label) in [("cover", "填充裁剪 (Cover)"), ("contain", "等比适应 (Fit)"), ("stretch", "拉伸铺满 (Stretch)")] {
                                {
                                    let is_active = current_fit == mode_key;
                                    let btn_style = if is_active {
                                        if is_dark {
                                            "background: #A8C7FA; color: #041E49; font-weight: 600;"
                                        } else {
                                            "background: #0B57D0; color: #FFFFFF; font-weight: 600;"
                                        }
                                    } else {
                                        "background: transparent; color: inherit; opacity: 0.75;"
                                    };
                                    let mode_str = mode_key.to_string();
                                    rsx! {
                                        button {
                                            key: "{mode_key}",
                                            onclick: move |_| on_change_fit.call(mode_str.clone()),
                                            style: "border: none; outline: none; cursor: pointer; padding: 4px 12px; border-radius: 9999px; font-size: 12px; transition: all 0.2s ease; {btn_style}",
                                            "{mode_label}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(ref path) = current_path {
                        div {
                            style: "display: flex; align-items: center; gap: 8px; font-size: 12px; opacity: 0.75;",
                            if let Some(ref t) = current_thumb {
                                img {
                                    src: "{t}",
                                    style: "width: 32px; height: 18px; border-radius: 4px; object-fit: cover;"
                                }
                            }
                            span {
                                style: "max-width: 340px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                "当前：{path.split('/').last().unwrap_or(path)}"
                            }
                        }
                    }
                }
            }

            // 2. 系统精选壁纸库（严谨的响应式网格，杜绝任何图层塌陷或重叠）
            div {
                class: "m3-card m3-card-bg",
                style: "display: flex; flex-direction: column;",

                // 标题栏：展示当前自适应屏幕匹配状态
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; flex-wrap: wrap; gap: 8px;",
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        span { style: "font-size: 16px;", "🖼️" }
                        h4 { style: "margin: 0; font-size: 15px; font-weight: 700;", "系统推荐壁纸库" }
                        span { style: "font-size: 12px; opacity: 0.55;", "(共 {system_wallpapers.len()} 张)" }
                    }

                    span {
                        class: "pulse-badge",
                        style: "background: rgba(14, 165, 233, 0.15); color: #0EA5E9; border: 1px solid rgba(14, 165, 233, 0.3); font-size: 11px;",
                        "📐 屏幕比例匹配: {m.id} ({m.width}×{m.height} · {ratio_name})"
                    }
                }

                // 严格独立单元格的 Grid 布局
                div {
                    style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 16px; max-height: 480px; min-height: 240px; overflow-y: auto; padding-right: 6px;",
                    for path in system_wallpapers.iter() {
                        {
                            let is_selected = current_path.as_ref() == Some(path);
                            let thumb = thumbnails.get(path).cloned();
                            let filename = path.split('/').last().unwrap_or(path).to_string();
                            let target_path = path.clone();

                            let card_border = if is_selected {
                                if is_dark {
                                    "2.5px solid #A8C7FA; box-shadow: 0 0 0 3px rgba(168, 199, 250, 0.25);"
                                } else {
                                    "2.5px solid #0B57D0; box-shadow: 0 0 0 3px rgba(11, 87, 208, 0.25);"
                                }
                            } else {
                                "1px solid rgba(140, 140, 150, 0.2);"
                            };

                            let object_fit_style = match current_fit.as_str() {
                                "contain" => "contain",
                                "stretch" => "fill",
                                _ => "cover",
                            };

                            rsx! {
                                div {
                                    key: "{path}",
                                    onclick: move |_| on_select_wallpaper.call(target_path.clone()),
                                    style: "position: relative; width: 100%; aspect-ratio: {ratio_w} / {ratio_h}; min-height: 120px; border-radius: 16px; overflow: hidden; cursor: pointer; background: rgba(140, 140, 150, 0.1); transition: all 0.2s cubic-bezier(0.2, 0, 0, 1); {card_border}",

                                    // 背景壁纸缩略图（使用 position: absolute 贴满自身卡片单元格，绝不影响 Grid 流）
                                    if let Some(t) = thumb {
                                        img {
                                            src: "{t}",
                                            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; object-fit: {object_fit_style}; display: block; background: #0E0E12; z-index: 1;"
                                        }
                                    } else {
                                        div {
                                            class: "skeleton-loading",
                                            style: "position: absolute; top: 0; left: 0; width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; font-size: 11px; opacity: 0.5; background: rgba(140, 140, 150, 0.15); z-index: 1;",
                                            "🖼️ 渲染中..."
                                        }
                                    }

                                    // 底部文件名浮层
                                    div {
                                        style: "position: absolute; bottom: 0; left: 0; right: 0; padding: 6px 10px; background: linear-gradient(0deg, rgba(0,0,0,0.85) 0%, rgba(0,0,0,0) 100%); font-size: 11px; color: #FFF; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; z-index: 2;",
                                        "{filename}"
                                    }

                                    if is_selected {
                                        div {
                                            style: "position: absolute; top: 8px; right: 8px; background: #0B57D0; color: #FFF; font-size: 10px; font-weight: 700; padding: 2px 8px; border-radius: 9999px; box-shadow: 0 2px 6px rgba(0,0,0,0.4); z-index: 3;",
                                            "当前选用"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
