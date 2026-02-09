use std::{
    env,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use collector::{
    clear_stats, new_collector_state, set_paused, snapshot, start_collector, StatsSnapshot,
};
use tauri::{
    menu::{Menu, MenuItem, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, Wry,
};
use tauri_plugin_opener::OpenerExt;

mod collector;

struct AppState {
    inner: Arc<Mutex<collector::CollectorState>>,
}

type AppMenuItem = MenuItem<Wry>;

struct TraySummaryItems {
    status_item: AppMenuItem,
    keyboard_item: AppMenuItem,
    toggle_item: AppMenuItem,
    typing_item: AppMenuItem,
    key_item: AppMenuItem,
    session_item: AppMenuItem,
}

#[tauri::command]
fn get_snapshot(state: tauri::State<AppState>) -> StatsSnapshot {
    if let Ok(locked) = state.inner.lock() {
        return snapshot(&locked);
    }
    StatsSnapshot {
        rows: vec![],
        paused: false,
        keyboard_active: false,
        last_error: Some("state lock failed".to_string()),
        log_path: "".to_string(),
    }
}

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

#[tauri::command]
fn reset_stats(state: tauri::State<AppState>) -> StatsSnapshot {
    if let Ok(mut locked) = state.inner.lock() {
        clear_stats(&mut locked);
        let _ = collector::append_app_log(&locked.app_log_path, "stats reset");
        return snapshot(&locked);
    }
    get_snapshot(state)
}

#[tauri::command]
fn get_log_path(state: tauri::State<AppState>) -> String {
    if let Ok(locked) = state.inner.lock() {
        return locked.log_path.to_string_lossy().to_string();
    }
    "".to_string()
}

#[tauri::command]
fn get_app_log_path(state: tauri::State<AppState>) -> String {
    if let Ok(locked) = state.inner.lock() {
        return locked.app_log_path.to_string_lossy().to_string();
    }
    "".to_string()
}

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            let _ = collector::append_app_log(&app_log_path, "app started");
            let panic_log_path = app_log_path.clone();
            std::panic::set_hook(Box::new(move |info| {
                let _ = collector::append_app_log(&panic_log_path, &format!("panic: {}", info));
            }));
            let state = Arc::new(Mutex::new(new_collector_state(
                log_path,
                app_log_path,
                detail_path,
            )));
            start_collector(state.clone());
            app.manage(AppState {
                inner: state.clone(),
            });
            let tray_items = build_tray(app)?;
            start_tray_updater(state, tray_items);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            update_paused,
            reset_stats,
            get_log_path,
            get_app_log_path,
            get_log_tail,
            get_app_log_tail,
            open_data_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_tray(app: &tauri::App) -> tauri::Result<TraySummaryItems> {
    let status_item = MenuItemBuilder::with_id("status", "采集状态: 运行中")
        .enabled(false)
        .build(app)?;
    let keyboard_item = MenuItemBuilder::with_id("keyboard", "键盘监听: 已启用")
        .enabled(false)
        .build(app)?;
    let toggle_item = MenuItemBuilder::with_id("toggle", "暂停采集")
        .enabled(true)
        .build(app)?;
    let typing_item = MenuItemBuilder::with_id("typing", "打字时长: 0m 0s")
        .enabled(false)
        .build(app)?;
    let key_item = MenuItemBuilder::with_id("keys", "按键次数: 0")
        .enabled(false)
        .build(app)?;
    let session_item = MenuItemBuilder::with_id("sessions", "会话次数: 0")
        .enabled(false)
        .build(app)?;
    let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let separator_top = PredefinedMenuItem::separator(app)?;
    let separator_bottom = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &status_item,
            &keyboard_item,
            &toggle_item,
            &separator_top,
            &typing_item,
            &key_item,
            &session_item,
            &separator_bottom,
            &show_item,
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

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon).icon_as_template(true);
    }

    builder.build(app)?;

    Ok(TraySummaryItems {
        status_item,
        keyboard_item,
        toggle_item,
        typing_item,
        key_item,
        session_item,
    })
}

fn start_tray_updater(state: Arc<Mutex<collector::CollectorState>>, items: TraySummaryItems) {
    let _ = update_tray_summary(&items, &get_snapshot_from_state(&state));
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        let snapshot = get_snapshot_from_state(&state);
        let _ = update_tray_summary(&items, &snapshot);
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
        last_error: Some("state lock failed".to_string()),
        log_path: "".to_string(),
    }
}

fn update_tray_summary(items: &TraySummaryItems, snapshot: &StatsSnapshot) -> tauri::Result<()> {
    let (active, keys, sessions) = snapshot
        .rows
        .iter()
        .fold((0u64, 0u64, 0u64), |mut acc, row| {
            acc.0 += row.active_typing_ms;
            acc.1 += row.key_count;
            acc.2 += row.session_count;
            acc
        });

    let status_text = if snapshot.paused {
        "采集状态: 已暂停".to_string()
    } else {
        "采集状态: 运行中".to_string()
    };
    let keyboard_text = if snapshot.keyboard_active {
        "键盘监听: 已启用".to_string()
    } else {
        "键盘监听: 未启用".to_string()
    };
    let toggle_text = if snapshot.paused {
        "继续采集".to_string()
    } else {
        "暂停采集".to_string()
    };

    items.status_item.set_text(status_text)?;
    items.keyboard_item.set_text(keyboard_text)?;
    items.toggle_item.set_text(toggle_text)?;
    items
        .typing_item
        .set_text(format!("打字时长: {}", format_ms(active)))?;
    items.key_item.set_text(format!("按键次数: {}", keys))?;
    items
        .session_item
        .set_text(format!("会话次数: {}", sessions))?;
    Ok(())
}

fn format_ms(ms: u64) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{}m {}s", minutes, seconds)
}
