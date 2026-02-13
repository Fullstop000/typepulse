use std::{fs, path::PathBuf};

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::{
    app_config::{save_app_config, MenuBarDisplayMode},
    apply_menu_bar_mode_immediately,
    collector::{
        self, bundle_id_from_app_path, running_apps, snapshot_shortcut_rows_by_range,
        snapshot_top_keys_by_range, KeyUsageRow, RunningAppInfo, ShortcutStatRow,
        StatsSnapshot,
    },
    AppState,
};

/// 获取当前采集快照，供前端轮询刷新仪表盘。
#[tauri::command]
pub(crate) fn get_snapshot(state: State<AppState>) -> StatsSnapshot {
    if let Ok(locked) = state.inner.lock() {
        return locked.snapshot();
    }
    StatsSnapshot {
        rows: vec![],
        paused: false,
        auto_paused: false,
        auto_pause_reason: None,
        keyboard_active: false,
        ignore_key_combos: false,
        excluded_bundle_ids: vec![],
        one_password_suggestion_pending: false,
        tray_display_mode: MenuBarDisplayMode::default().as_str().to_string(),
        last_error: Some("state lock failed".to_string()),
        log_path: "".to_string(),
        shortcut_stats: vec![],
    }
}

/// 按时间范围返回快捷键排行榜（today / yesterday / 7d）。
#[tauri::command]
pub(crate) fn get_shortcut_stats_by_range(
    state: State<AppState>,
    range: String,
) -> Vec<ShortcutStatRow> {
    if let Ok(locked) = state.inner.lock() {
        return snapshot_shortcut_rows_by_range(&locked, &range);
    }
    vec![]
}

/// 按时间范围返回 Top5 按键（today / yesterday / 7d，聚合展示）。
#[tauri::command]
pub(crate) fn get_daily_top_keys_by_range(
    state: State<AppState>,
    range: String,
) -> Vec<KeyUsageRow> {
    if let Ok(locked) = state.inner.lock() {
        return snapshot_top_keys_by_range(&locked, &range);
    }
    vec![]
}

/// 更新采集暂停状态，并返回最新快照。
#[tauri::command]
pub(crate) fn update_paused(state: State<AppState>, paused: bool) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        locked.set_paused(paused);
        let _ = collector::append_app_log(
            &locked.app_log_path,
            if paused {
                "paused via command"
            } else {
                "resumed via command"
            },
        );
        return locked.snapshot();
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
        locked.set_ignore_key_combos(ignore_key_combos);
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
        return locked.snapshot();
    }
    get_snapshot(state)
}

/// 更新快捷键统计规则配置并返回最新快照。
#[tauri::command]
pub(crate) fn update_shortcut_rules(
    state: State<AppState>,
    require_cmd_or_ctrl: bool,
    allow_alt_only: bool,
    min_modifiers: u8,
    allowlist: Vec<String>,
    blocklist: Vec<String>,
) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        locked.set_shortcut_rules(
            require_cmd_or_ctrl,
            allow_alt_only,
            min_modifiers,
            &allowlist,
            &blocklist,
        );
        if let Ok(mut config) = state.config.lock() {
            config.shortcut_require_cmd_or_ctrl = require_cmd_or_ctrl;
            config.shortcut_allow_alt_only = allow_alt_only;
            config.shortcut_min_modifiers = min_modifiers.max(1);
            config.shortcut_allowlist = allowlist
                .iter()
                .map(|v| v.trim().to_ascii_lowercase())
                .filter(|v| !v.is_empty())
                .collect();
            config.shortcut_allowlist.sort();
            config.shortcut_allowlist.dedup();
            config.shortcut_blocklist = blocklist
                .iter()
                .map(|v| v.trim().to_ascii_lowercase())
                .filter(|v| !v.is_empty())
                .collect();
            config.shortcut_blocklist.sort();
            config.shortcut_blocklist.dedup();
            let _ = save_app_config(&state.config_path, &config);
        }
        let _ = collector::append_app_log(&locked.app_log_path, "shortcut rules updated");
        return locked.snapshot();
    }
    get_snapshot(state)
}

#[tauri::command]
pub(crate) fn get_running_apps() -> Vec<RunningAppInfo> {
    running_apps()
}

#[tauri::command]
pub(crate) fn update_app_exclusion_list(
    state: State<AppState>,
    bundle_ids: Vec<String>,
) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        locked.set_excluded_bundle_ids(&bundle_ids);
        if let Ok(mut config) = state.config.lock() {
            config.excluded_bundle_ids = bundle_ids
                .iter()
                .map(|v| v.trim().to_ascii_lowercase())
                .filter(|v| !v.is_empty())
                .collect();
            let _ = save_app_config(&state.config_path, &config);
        }
        let _ = collector::append_app_log(&locked.app_log_path, "app exclusion list updated");
        return locked.snapshot();
    }
    get_snapshot(state)
}

#[tauri::command]
pub(crate) fn add_app_exclusion(state: State<AppState>, bundle_id: String) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        let added = locked.add_excluded_bundle_id(&bundle_id);
        if added {
            if let Ok(mut config) = state.config.lock() {
                let normalized = bundle_id.trim().to_ascii_lowercase();
                if !config
                    .excluded_bundle_ids
                    .iter()
                    .any(|v| v.eq_ignore_ascii_case(&normalized))
                {
                    config.excluded_bundle_ids.push(normalized.clone());
                }
                config.excluded_bundle_ids.sort();
                config.excluded_bundle_ids.dedup();
                let _ = save_app_config(&state.config_path, &config);
            }
            let _ = collector::append_app_log(
                &locked.app_log_path,
                &format!("bundle id added to exclusion list: {}", bundle_id),
            );
        }
        return locked.snapshot();
    }
    get_snapshot(state)
}

#[tauri::command]
pub(crate) fn remove_app_exclusion(state: State<AppState>, bundle_id: String) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        let removed = locked.remove_excluded_bundle_id(&bundle_id);
        if removed {
            if let Ok(mut config) = state.config.lock() {
                config
                    .excluded_bundle_ids
                    .retain(|v| !v.eq_ignore_ascii_case(bundle_id.as_str()));
                let _ = save_app_config(&state.config_path, &config);
            }
            let _ = collector::append_app_log(
                &locked.app_log_path,
                &format!("bundle id removed from exclusion list: {}", bundle_id),
            );
        }
        return locked.snapshot();
    }
    get_snapshot(state)
}

#[tauri::command]
pub(crate) fn resolve_bundle_id_from_app_path(path: String) -> Option<String> {
    bundle_id_from_app_path(&path)
}

#[tauri::command]
pub(crate) fn dismiss_one_password_suggestion(state: State<AppState>) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        locked.set_one_password_suggestion_pending(false);
        if let Ok(mut config) = state.config.lock() {
            config.one_password_suggestion_handled = true;
            let _ = save_app_config(&state.config_path, &config);
        }
        return locked.snapshot();
    }
    get_snapshot(state)
}

#[tauri::command]
pub(crate) fn accept_one_password_suggestion(state: State<AppState>) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        let _ = locked.add_excluded_bundle_id("com.1password.1password");
        locked.set_one_password_suggestion_pending(false);
        if let Ok(mut config) = state.config.lock() {
            if !config
                .excluded_bundle_ids
                .iter()
                .any(|v| v.eq_ignore_ascii_case("com.1password.1password"))
            {
                config
                    .excluded_bundle_ids
                    .push("com.1password.1password".to_string());
            }
            config.excluded_bundle_ids.sort();
            config.excluded_bundle_ids.dedup();
            config.one_password_suggestion_handled = true;
            let _ = save_app_config(&state.config_path, &config);
        }
        let _ = collector::append_app_log(
            &locked.app_log_path,
            "1Password added to exclusion list via suggestion",
        );
        return locked.snapshot();
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
        locked.set_menu_bar_display_mode(mode);
        if let Ok(mut config) = state.config.lock() {
            config.menu_bar_display_mode = mode;
            let _ = save_app_config(&state.config_path, &config);
        }
        let _ = collector::append_app_log(
            &locked.app_log_path,
            &format!("menu bar display mode changed: {}", mode.as_str()),
        );
        let snapshot = locked.snapshot();
        apply_menu_bar_mode_immediately(&app, &snapshot);
        return snapshot;
    }
    get_snapshot(state)
}

/// 清空已采集统计数据并返回最新快照。
#[tauri::command]
pub(crate) fn reset_stats(state: State<AppState>) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        locked.clear_stats();
        let _ = collector::append_app_log(&locked.app_log_path, "stats reset");
        return locked.snapshot();
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
