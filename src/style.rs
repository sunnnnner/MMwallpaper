pub fn get_app_styles(is_dark: bool) -> &'static str {
    if is_dark {
        DARK_CSS
    } else {
        LIGHT_CSS
    }
}

const COMMON_CSS: &str = r#"
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
    user-select: none;
}

body {
    font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    -webkit-font-smoothing: antialiased;
    overflow-x: hidden;
}

::-webkit-scrollbar {
    width: 6px;
    height: 6px;
}

::-webkit-scrollbar-track {
    background: transparent;
}

::-webkit-scrollbar-thumb {
    background: rgba(140, 140, 150, 0.3);
    border-radius: 9999px;
}

::-webkit-scrollbar-thumb:hover {
    background: rgba(140, 140, 150, 0.6);
}

.m3-btn {
    border: none;
    outline: none;
    cursor: pointer;
    font-weight: 500;
    font-size: 13px;
    padding: 8px 18px;
    border-radius: 9999px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    transition: all 0.2s cubic-bezier(0.2, 0, 0, 1);
}

.m3-btn:active {
    transform: scale(0.97);
}

.m3-card {
    border-radius: 22px;
    padding: 20px 24px;
    transition: all 0.25s ease;
}

.pulse-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 9999px;
}

@keyframes skeleton-shimmer {
    0% { opacity: 0.35; }
    50% { opacity: 0.7; }
    100% { opacity: 0.35; }
}

.skeleton-loading {
    animation: skeleton-shimmer 1.5s infinite ease-in-out;
}

"#;

const DARK_CSS: &str = r#"
body {
    background-color: #121215;
    color: #E2E2EA;
}

.app-header {
    background-color: #1B1B22;
    border-bottom: 1px solid #2B2B38;
}

.m3-card-bg {
    background-color: #1E1E26;
    border: 1px solid #2F2F3D;
}

.m3-btn-primary {
    background-color: #A8C7FA;
    color: #041E49;
    box-shadow: 0 4px 14px rgba(168, 199, 250, 0.25);
}

.m3-btn-primary:hover {
    background-color: #BDD7FF;
}

.m3-btn-tonal {
    background-color: #272733;
    color: #D6D6E2;
}

.m3-btn-tonal:hover {
    background-color: #323242;
}

.m3-btn-danger {
    background-color: rgba(239, 68, 68, 0.15);
    color: #F87171;
    border: 1px solid rgba(239, 68, 68, 0.3);
}

.m3-btn-danger:hover {
    background-color: rgba(239, 68, 68, 0.25);
}

.monitor-canvas-bg {
    background-color: #17171C;
    border: 1px solid #2B2B36;
}
"#;

const LIGHT_CSS: &str = r#"
body {
    background-color: #F8F9FA;
    color: #1C1B1F;
}

.app-header {
    background-color: #FFFFFF;
    border-bottom: 1px solid #E8E8EF;
}

.m3-card-bg {
    background-color: #FFFFFF;
    border: 1px solid #E4E4EB;
    box-shadow: 0 1px 4px rgba(0,0,0,0.04);
}

.m3-btn-primary {
    background-color: #0B57D0;
    color: #FFFFFF;
    box-shadow: 0 4px 14px rgba(11, 87, 208, 0.2);
}

.m3-btn-primary:hover {
    background-color: #0842A0;
}

.m3-btn-tonal {
    background-color: #EBEBF2;
    color: #303038;
}

.m3-btn-tonal:hover {
    background-color: #DFDFE8;
}

.m3-btn-danger {
    background-color: #FEE2E2;
    color: #B91C1C;
    border: 1px solid #FECACA;
}

.m3-btn-danger:hover {
    background-color: #FCA5A5;
}

.monitor-canvas-bg {
    background-color: #F3F3F7;
    border: 1px solid #E5E5EB;
}
"#;

pub fn build_full_css(is_dark: bool) -> String {
    format!("{}\n{}", COMMON_CSS, get_app_styles(is_dark))
}
