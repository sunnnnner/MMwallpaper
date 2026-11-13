use dioxus::prelude::*;
use crate::models::MonitorInfo;

#[component]
pub fn PreviewModal(
    open: bool,
    preview_base64: Option<String>,
    output_path: Option<String>,
    monitors: Vec<MonitorInfo>,
    is_dark: bool,
    on_close: EventHandler<()>,
) -> Element {
    if !open {
        return rsx! { div {} };
    }

    let modal_bg = if is_dark { "#1B1B22" } else { "#FFFFFF" };
    let border_color = if is_dark { "#33333F" } else { "#E2E2EA" };

    rsx! {
        // 半透明遮罩
        div {
            onclick: move |_| on_close.call(()),
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.65); backdrop-filter: blur(4px); z-index: 1000; display: flex; align-items: center; justify-content: center; padding: 24px;",

            // 模态弹窗主体
            div {
                onclick: move |e| e.stop_propagation(),
                style: "background: {modal_bg}; border: 1px solid {border_color}; border-radius: 28px; width: 100%; max-width: 860px; max-height: 85vh; display: flex; flex-direction: column; overflow: hidden; box-shadow: 0 16px 40px rgba(0,0,0,0.4);",

                // 标题栏
                div {
                    style: "padding: 18px 24px; border-bottom: 1px solid rgba(140, 140, 150, 0.15); display: flex; justify-content: space-between; align-items: center;",
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        span { style: "font-size: 18px;", "👁️" }
                        h3 { style: "margin: 0; font-size: 16px; font-weight: 700;", "多屏幕跨屏合成全景预览" }
                    }
                    button {
                        class: "m3-btn m3-btn-tonal",
                        onclick: move |_| on_close.call(()),
                        style: "padding: 4px 14px; font-size: 12px;",
                        "关闭"
                    }
                }

                // 内容区
                div {
                    style: "padding: 20px 24px; overflow-y: auto; display: flex; flex-direction: column; gap: 14px;",
                    p {
                        style: "margin: 0; font-size: 13px; opacity: 0.7; line-height: 1.6;",
                        "以下是由 Rust 图像引擎按各显示器物理几何坐标拼接的完整巨幅壁纸，已通过 GNOME 原生 Spanned 模式无缝铺满桌面。"
                    }

                    // 屏幕分辨率小标签
                    div {
                        style: "display: flex; gap: 8px; flex-wrap: wrap;",
                        for m in monitors.iter() {
                            span {
                                class: "pulse-badge",
                                style: "background: rgba(14, 165, 233, 0.15); color: #0EA5E9; border: 1px solid rgba(14, 165, 233, 0.3);",
                                "{m.name}: {m.width}×{m.height}"
                            }
                        }
                    }

                    // 预览大图
                    div {
                        style: "width: 100%; min-height: 240px; max-height: 420px; border-radius: 18px; overflow: hidden; background: #0E0E12; border: 1px solid rgba(140, 140, 150, 0.2); display: flex; align-items: center; justify-content: center;",
                        if let Some(ref b64) = preview_base64 {
                            img {
                                src: "{b64}",
                                style: "max-width: 100%; max-height: 420px; object-fit: contain;"
                            }
                        } else {
                            span { style: "font-size: 13px; opacity: 0.5;", "暂无合成预览" }
                        }
                    }

                    if let Some(ref path) = output_path {
                        div {
                            style: "padding: 10px 14px; border-radius: 12px; background: rgba(140, 140, 150, 0.1); font-size: 12px; display: flex; justify-content: space-between; align-items: center;",
                            span {
                                style: "opacity: 0.8; word-break: break-all;",
                                "生成文件：{path}"
                            }
                            span {
                                class: "pulse-badge",
                                style: "background: rgba(16, 185, 129, 0.15); color: #10B981;",
                                "PNG Spanned"
                            }
                        }
                    }
                }
            }
        }
    }
}
