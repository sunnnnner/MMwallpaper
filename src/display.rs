use std::collections::HashMap;
use std::process::Command;
use crate::models::MonitorInfo;

/// 获取 Mutter DBus 中显示器 ID 到友好型号名称的映射
fn get_mutter_display_names() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Mutter.DisplayConfig",
            "--object-path",
            "/org/gnome/Mutter/DisplayConfig",
            "--method",
            "org.gnome.Mutter.DisplayConfig.GetCurrentState",
        ])
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        let parts: Vec<&str> = text.split("(('").collect();
        for p in parts.iter().skip(1) {
            if let Some(id_end) = p.find('\'') {
                let id = &p[..id_end];
                if let Some(name_pos) = p.find("'display-name': <'") {
                    let after = &p[name_pos + 17..];
                    if let Some(end_name) = after.find("'>") {
                        let name = &after[..end_name];
                        let clean_name = name.trim_matches('\'').trim_matches('"');
                        map.insert(id.to_string(), clean_name.to_string());
                    }
                }
            }
        }
    }
    map
}

/// 解析 xrandr --listmonitors 输出
pub fn detect_monitors() -> Result<Vec<MonitorInfo>, String> {
    let output = Command::new("xrandr")
        .arg("--listmonitors")
        .output()
        .map_err(|e| format!("Failed to execute xrandr: {}", e))?;

    if !output.status.success() {
        return Err("xrandr returned non-zero exit code".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let friendly_names = get_mutter_display_names();
    let mut monitors = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Monitors:") || trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let is_primary = parts[1].contains('*');
        let id = parts.last().unwrap_or(&"Unknown").to_string();

        let mut geom_str = None;
        for part in &parts {
            if part.contains('/') && part.contains('x') && part.contains('+') {
                geom_str = Some(*part);
                break;
            }
        }

        let geom = match geom_str {
            Some(g) => g,
            None => continue,
        };

        let clean_geom = geom.trim_start_matches('+').trim_start_matches('*');
        let x_split: Vec<&str> = clean_geom.split('x').collect();
        if x_split.len() != 2 {
            continue;
        }

        let width_part = x_split[0];
        let width: u32 = width_part.split('/').next().unwrap_or("1920").parse().unwrap_or(1920);

        let rest = x_split[1];
        let plus_split: Vec<&str> = rest.split('+').collect();
        if plus_split.len() < 3 {
            continue;
        }

        let height_part = plus_split[0];
        let height: u32 = height_part.split('/').next().unwrap_or("1080").parse().unwrap_or(1080);
        let x_offset: i32 = plus_split[1].parse().unwrap_or(0);
        let y_offset: i32 = plus_split[2].parse().unwrap_or(0);

        let friendly_name = if let Some(fn_name) = friendly_names.get(&id) {
            format!("{} ({})", fn_name, id)
        } else {
            format!("显示器 {}", id)
        };

        monitors.push(MonitorInfo {
            id,
            name: friendly_name,
            width,
            height,
            x: x_offset,
            y: y_offset,
            scale: 1.0,
            is_primary,
        });
    }

    if monitors.is_empty() {
        return Err("未通过 xrandr 检测到活动显示器".to_string());
    }

    monitors.sort_by_key(|m| m.x);
    Ok(monitors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_monitors() {
        let res = detect_monitors();
        assert!(res.is_ok());
        let list = res.unwrap();
        assert_eq!(list.len(), 2);
    }
}
