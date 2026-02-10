use std::{
    env, fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use app_config::{load_app_config, save_app_config, AppConfig, MenuBarDisplayMode};
use chrono::Local;
use collector::{
    clear_stats, new_collector_state, set_ignore_key_combos, set_menu_bar_display_mode, set_paused,
    snapshot, start_collector, StatsSnapshot,
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, Wry,
};
use tauri_plugin_opener::OpenerExt;

mod app_config;
mod collector;
mod storage;

struct AppState {
    inner: Arc<Mutex<collector::CollectorState>>,
    config: Arc<Mutex<AppConfig>>,
    config_path: PathBuf,
}

type AppMenuItem = MenuItem<Wry>;

struct TraySummaryItems {
    tray_icon: tauri::tray::TrayIcon<Wry>,
    black_icon: Option<Image<'static>>,
    overview_item: AppMenuItem,
    toggle_item: AppMenuItem,
}

/// 获取当前采集快照，供前端轮询刷新仪表盘。
#[tauri::command]
fn get_snapshot(state: tauri::State<AppState>) -> StatsSnapshot {
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
fn update_paused(state: tauri::State<AppState>, paused: bool) -> StatsSnapshot {
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
fn update_ignore_key_combos(
    state: tauri::State<AppState>,
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
fn update_menu_bar_display_mode(
    state: tauri::State<AppState>,
    app: tauri::AppHandle,
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
fn reset_stats(state: tauri::State<AppState>) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        clear_stats(&mut locked);
        let _ = collector::append_app_log(&locked.app_log_path, "stats reset");
        return snapshot(&locked);
    }
    get_snapshot(state)
}

/// 获取汇总日志（CSV）文件路径。
#[tauri::command]
fn get_log_path(state: tauri::State<AppState>) -> String {
    if let Ok(locked) = state.inner.lock() {
        return locked.log_path.to_string_lossy().to_string();
    }
    "".to_string()
}

/// 获取应用运行日志文件路径。
#[tauri::command]
fn get_app_log_path(state: tauri::State<AppState>) -> String {
    if let Ok(locked) = state.inner.lock() {
        return locked.app_log_path.to_string_lossy().to_string();
    }
    "".to_string()
}

/// 获取汇总日志末尾内容（最多近 200 行）。
#[tauri::command]
fn get_log_tail(state: tauri::State<AppState>) -> String {
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
fn get_app_log_tail(state: tauri::State<AppState>) -> String {
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
fn open_data_dir(state: tauri::State<AppState>, app: tauri::AppHandle) -> Result<(), String> {
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
fn get_data_dir_size(state: tauri::State<AppState>) -> u64 {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .setup(|app| {
            let data_dir = if cfg!(debug_assertions) {
                env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join("_data")
            } else {
                app.path()
                    .app_data_dir()
                    .or_else(|_| env::current_dir())
                    .unwrap_or_else(|_| PathBuf::from("."))
            };
            let _ = std::fs::create_dir_all(&data_dir);
            let log_path = data_dir.join("typingstats.csv");
            let app_log_path = data_dir.join("typingstats-app.log");
            let detail_path = data_dir.join("typingstats-details.json");
            let config_path = data_dir.join("typingstats-config.json");
            let config = load_app_config(&config_path).unwrap_or_default();
            let tray_update_interval = config.tray_update_interval();
            let _ = collector::append_app_log(&app_log_path, "app started");
            let panic_log_path = app_log_path.clone();
            std::panic::set_hook(Box::new(move |info| {
                let _ = collector::append_app_log(&panic_log_path, &format!("panic: {}", info));
            }));
            let state = Arc::new(Mutex::new(new_collector_state(
                log_path,
                app_log_path,
                detail_path,
                &config,
            )));
            start_collector(state.clone());
            app.manage(AppState {
                inner: state.clone(),
                config: Arc::new(Mutex::new(config)),
                config_path,
            });
            let tray_items = build_tray(app)?;
            start_tray_updater(state, tray_items, tray_update_interval);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            update_paused,
            update_ignore_key_combos,
            update_menu_bar_display_mode,
            reset_stats,
            get_log_path,
            get_app_log_path,
            get_log_tail,
            get_app_log_tail,
            open_data_dir,
            get_data_dir_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_tray(app: &tauri::App) -> tauri::Result<TraySummaryItems> {
    let overview_item = MenuItemBuilder::with_id("overview", "今日时长: 0h 0m | 今日总键数: 0")
        .enabled(false)
        .build(app)?;
    let toggle_item = MenuItemBuilder::with_id("toggle", "暂停采集")
        .enabled(true)
        .build(app)?;
    let show_item = MenuItem::with_id(app, "show", "打开主面板", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let separator_middle = PredefinedMenuItem::separator(app)?;
    let separator_bottom = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &overview_item,
            &toggle_item,
            &separator_middle,
            &show_item,
            &separator_bottom,
            &quit_item,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .tooltip("TypePulse")
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            if event.id() == "show" {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            if event.id() == "quit" {
                app.exit(0);
            }
            if event.id() == "toggle" {
                if let Ok(mut locked) = app.state::<AppState>().inner.lock() {
                    let paused_now = snapshot(&locked).paused;
                    set_paused(&mut locked, !paused_now);
                    let _ = collector::append_app_log(
                        &locked.app_log_path,
                        if paused_now {
                            "resumed via tray"
                        } else {
                            "paused via tray"
                        },
                    );
                }
            }
        });

    let black_icon = Image::from_bytes(include_bytes!("../icons/l_black.png"))
        .ok()
        .or_else(|| app.default_window_icon().cloned())
        .map(Image::to_owned);
    if let Some(icon) = black_icon.clone() {
        builder = builder.icon(icon);
    }
    #[cfg(target_os = "macos")]
    {
        builder = builder.icon_as_template(false);
    }

    let tray_icon = builder.build(app)?;

    Ok(TraySummaryItems {
        tray_icon,
        black_icon,
        overview_item,
        toggle_item,
    })
}

fn start_tray_updater(
    state: Arc<Mutex<collector::CollectorState>>,
    items: TraySummaryItems,
    tick_interval: std::time::Duration,
) {
    let mut last_total_keys = 0u64;
    let mut last_title: Option<String> = None;
    let mut last_mode = MenuBarDisplayMode::default();
    let _ = update_tray_summary(
        &items,
        &get_snapshot_from_state(&state),
        &mut last_total_keys,
        &mut last_title,
        &mut last_mode,
    );
    std::thread::spawn(move || loop {
        std::thread::sleep(tick_interval);
        let snapshot = get_snapshot_from_state(&state);
        let _ = update_tray_summary(
            &items,
            &snapshot,
            &mut last_total_keys,
            &mut last_title,
            &mut last_mode,
        );
    });
}

fn get_snapshot_from_state(state: &Arc<Mutex<collector::CollectorState>>) -> StatsSnapshot {
    if let Ok(locked) = state.lock() {
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

fn update_tray_summary(
    items: &TraySummaryItems,
    snapshot: &StatsSnapshot,
    last_total_keys: &mut u64,
    last_title: &mut Option<String>,
    last_mode: &mut MenuBarDisplayMode,
) -> tauri::Result<()> {
    let today_prefix = Local::now().format("%Y-%m-%d").to_string();
    let (active, keys) = snapshot
        .rows
        .iter()
        .filter(|row| row.date.starts_with(&today_prefix))
        .fold((0u64, 0u64), |mut acc, row| {
            acc.0 += row.active_typing_ms;
            acc.1 += row.key_count;
            acc
        });

    let mode = MenuBarDisplayMode::from_str(&snapshot.tray_display_mode).unwrap_or_default();
    let toggle_text = if snapshot.paused {
        "继续采集".to_string()
    } else {
        "暂停采集".to_string()
    };
    let compact_keys = format_compact_number(keys);
    let title = match mode {
        MenuBarDisplayMode::IconOnly => Some(String::new()),
        MenuBarDisplayMode::TextOnly | MenuBarDisplayMode::IconText => Some(compact_keys.clone()),
    };
    let should_update_icon = mode != *last_mode;
    let should_update_title = mode != *last_mode || title != *last_title;
    if should_update_icon {
        match mode {
            MenuBarDisplayMode::TextOnly => {
                let _ = items.tray_icon.set_icon(None);
            }
            MenuBarDisplayMode::IconOnly | MenuBarDisplayMode::IconText => {
                let _ = items.tray_icon.set_icon(items.black_icon.clone());
            }
        }
    }
    if should_update_title {
        let _ = items.tray_icon.set_title(title.clone());
    }

    items.overview_item.set_text(format!(
        "今日时长: {} | 今日总键数: {}",
        format_hm(active),
        compact_keys
    ))?;
    items.toggle_item.set_text(toggle_text)?;

    *last_total_keys = keys;
    *last_title = title;
    *last_mode = mode;
    Ok(())
}

fn format_hm(ms: u64) -> String {
    let total_minutes = ms / 1000 / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{}h {}m", hours, minutes)
}

fn format_compact_number(value: u64) -> String {
    if value < 1_000 {
        return value.to_string();
    }
    if value < 1_000_000 {
        return format_one_decimal(value as f64 / 1_000f64, "k");
    }
    if value < 1_000_000_000 {
        return format_one_decimal(value as f64 / 1_000_000f64, "m");
    }
    format_one_decimal(value as f64 / 1_000_000_000f64, "b")
}

fn format_one_decimal(base: f64, suffix: &str) -> String {
    let rounded = (base * 10.0).round() / 10.0;
    if (rounded - rounded.trunc()).abs() < f64::EPSILON {
        format!("{}{}", rounded as u64, suffix)
    } else {
        format!("{:.1}{}", rounded, suffix)
    }
}

fn apply_menu_bar_mode_immediately(app: &tauri::AppHandle, snapshot: &StatsSnapshot) {
    let Some(tray) = app.tray_by_id("main-tray") else {
        return;
    };
    let today_prefix = Local::now().format("%Y-%m-%d").to_string();
    let keys = snapshot
        .rows
        .iter()
        .filter(|row| row.date.starts_with(&today_prefix))
        .fold(0u64, |acc, row| acc + row.key_count);
    let compact_keys = format_compact_number(keys);
    let mode = MenuBarDisplayMode::from_str(&snapshot.tray_display_mode).unwrap_or_default();
    match mode {
        MenuBarDisplayMode::IconOnly => {
            let _ = tray.set_title(Some(String::new()));
            let icon = Image::from_bytes(include_bytes!("../icons/l_black.png"))
                .ok()
                .or_else(|| app.default_window_icon().cloned());
            let _ = tray.set_icon(icon);
        }
        MenuBarDisplayMode::TextOnly => {
            let _ = tray.set_icon(None);
            let _ = tray.set_title(Some(compact_keys));
        }
        MenuBarDisplayMode::IconText => {
            let icon = Image::from_bytes(include_bytes!("../icons/l_black.png"))
                .ok()
                .or_else(|| app.default_window_icon().cloned());
            let _ = tray.set_icon(icon);
            let _ = tray.set_title(Some(compact_keys));
        }
    }
}
