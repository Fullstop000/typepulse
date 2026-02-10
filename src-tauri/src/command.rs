use std::{fs, path::PathBuf};

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::{
    app_config::{save_app_config, MenuBarDisplayMode},
    apply_menu_bar_mode_immediately,
    collector::{
        self, clear_stats, set_ignore_key_combos, set_menu_bar_display_mode, set_paused, snapshot,
        StatsSnapshot,
    },
    AppState,
};

/// 获取当前采集快照，供前端轮询刷新仪表盘。
#[tauri::command]
pub(crate) fn get_snapshot(state: State<AppState>) -> StatsSnapshot {
    if let Ok(locked) = state.inner.lock() {
        return snapshot(&locked);
    }
    StatsSnapshot {
        rows: vec![],
        paused: false,
        keyboard_active: false,
        ignore_key_combos: false,
        tray_display_mode: MenuBarDisplayMode::default().as_str().to_string(),
        last_error: Some("state lock failed".to_string()),
        log_path: "".to_string(),
    }
}

/// 更新采集暂停状态，并返回最新快照。
#[tauri::command]
pub(crate) fn update_paused(state: State<AppState>, paused: bool) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        set_paused(&mut locked, paused);
        let _ = collector::append_app_log(
            &locked.app_log_path,
            if paused {
                "paused via command"
            } else {
                "resumed via command"
            },
        );
        return snapshot(&locked);
    }
    get_snapshot(state)
}

/// 切换“忽略组合键”设置，持久化配置后返回最新快照。
#[tauri::command]
pub(crate) fn update_ignore_key_combos(
    state: State<AppState>,
    ignore_key_combos: bool,
) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        set_ignore_key_combos(&mut locked, ignore_key_combos);
        if let Ok(mut config) = state.config.lock() {
            config.ignore_key_combos = ignore_key_combos;
            let _ = save_app_config(&state.config_path, &config);
        }
        let _ = collector::append_app_log(
            &locked.app_log_path,
            if ignore_key_combos {
                "ignore key combos enabled"
            } else {
                "ignore key combos disabled"
            },
        );
        return snapshot(&locked);
    }
    get_snapshot(state)
}

/// 更新菜单栏显示模式，立即应用到托盘并返回最新快照。
#[tauri::command]
pub(crate) fn update_menu_bar_display_mode(
    state: State<AppState>,
    app: AppHandle,
    mode: String,
) -> StatsSnapshot {
    let mode = match MenuBarDisplayMode::from_str(&mode) {
        Some(mode) => mode,
        None => return get_snapshot(state),
    };
    if let Ok(mut locked) = state.inner.lock() {
        set_menu_bar_display_mode(&mut locked, mode);
        if let Ok(mut config) = state.config.lock() {
            config.menu_bar_display_mode = mode;
            let _ = save_app_config(&state.config_path, &config);
        }
        let _ = collector::append_app_log(
            &locked.app_log_path,
            &format!("menu bar display mode changed: {}", mode.as_str()),
        );
        let snapshot = snapshot(&locked);
        apply_menu_bar_mode_immediately(&app, &snapshot);
        return snapshot;
    }
    get_snapshot(state)
}

/// 清空已采集统计数据并返回最新快照。
#[tauri::command]
pub(crate) fn reset_stats(state: State<AppState>) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        clear_stats(&mut locked);
        let _ = collector::append_app_log(&locked.app_log_path, "stats reset");
        return snapshot(&locked);
    }
    get_snapshot(state)
}

/// 获取汇总日志（CSV）文件路径。
#[tauri::command]
pub(crate) fn get_log_path(state: State<AppState>) -> String {
    if let Ok(locked) = state.inner.lock() {
        return locked.log_path.to_string_lossy().to_string();
    }
    "".to_string()
}

/// 获取应用运行日志文件路径。
#[tauri::command]
pub(crate) fn get_app_log_path(state: State<AppState>) -> String {
    if let Ok(locked) = state.inner.lock() {
        return locked.app_log_path.to_string_lossy().to_string();
    }
    "".to_string()
}

/// 获取汇总日志末尾内容（最多近 200 行）。
#[tauri::command]
pub(crate) fn get_log_tail(state: State<AppState>) -> String {
    let path = if let Ok(locked) = state.inner.lock() {
        locked.log_path.clone()
    } else {
        return "".to_string();
    };
    if let Ok(content) = std::fs::read_to_string(path) {
        let lines: Vec<&str> = content.lines().collect();
        let start = lines.len().saturating_sub(200);
        return lines[start..].join("\n");
    }
    "".to_string()
}

/// 获取应用日志末尾内容（最多近 400 行）。
#[tauri::command]
pub(crate) fn get_app_log_tail(state: State<AppState>) -> String {
    let path = if let Ok(locked) = state.inner.lock() {
        locked.app_log_path.clone()
    } else {
        return "".to_string();
    };
    if let Ok(content) = std::fs::read_to_string(path) {
        let lines: Vec<&str> = content.lines().collect();
        let start = lines.len().saturating_sub(400);
        return lines[start..].join("\n");
    }
    "".to_string()
}

/// 打开本地数据目录（日志与明细文件所在目录）。
#[tauri::command]
pub(crate) fn open_data_dir(state: State<AppState>, app: AppHandle) -> Result<(), String> {
    let path = if let Ok(locked) = state.inner.lock() {
        locked.log_path.clone()
    } else {
        return Err("state lock failed".to_string());
    };
    let data_dir = path.parent().unwrap_or(path.as_path());
    let _ = std::fs::create_dir_all(data_dir);
    app.opener()
        .open_path(data_dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|err| err.to_string())
}

/// 计算并返回数据目录总大小（字节）。
#[tauri::command]
pub(crate) fn get_data_dir_size(state: State<AppState>) -> u64 {
    let path = if let Ok(locked) = state.inner.lock() {
        locked.log_path.clone()
    } else {
        return 0;
    };
    let data_dir = path.parent().unwrap_or(path.as_path()).to_path_buf();
    let _ = fs::create_dir_all(&data_dir);
    folder_size(&data_dir)
}

fn folder_size(path: &PathBuf) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![path.clone()];
    while let Some(current) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            if metadata.is_dir() {
                stack.push(path);
            } else {
                total += metadata.len();
            }
        }
    }
    total
}
