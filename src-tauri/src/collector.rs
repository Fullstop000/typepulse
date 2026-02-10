use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chrono::Local;
use serde::Serialize;

use crate::app_config::{AppConfig, MenuBarDisplayMode};
use crate::storage::{DetailStorage, JsonFileStorage};

#[derive(Clone, Hash, Eq, PartialEq)]
pub(crate) struct StatsKey {
    pub(crate) date: String,
    pub(crate) app_name: String,
    pub(crate) window_title: String,
}

#[derive(Clone)]
pub(crate) struct StatsValue {
    pub(crate) active_typing_ms: u64,
    pub(crate) key_count: u64,
    pub(crate) session_count: u64,
}

#[derive(Serialize, Clone)]
pub struct StatsRow {
    pub date: String,
    pub app_name: String,
    pub window_title: String,
    pub active_typing_ms: u64,
    pub key_count: u64,
    pub session_count: u64,
}

#[derive(Serialize, Clone)]
pub struct StatsSnapshot {
    pub rows: Vec<StatsRow>,
    pub paused: bool,
    pub keyboard_active: bool,
    pub ignore_key_combos: bool,
    pub tray_display_mode: String,
    pub last_error: Option<String>,
    pub log_path: String,
}

pub struct CollectorState {
    // 统计聚合的明细数据（按时间/应用/窗口维度）
    stats: HashMap<StatsKey, StatsValue>,
    // 最近一次按键的时间点，用于计算间隔与会话
    last_key_instant: Instant,
    // 最近一次刷盘时间点，用于控制落盘频率
    last_flush_instant: Instant,
    // 采集线程轮询周期
    collector_tick_interval: Duration,
    // 统计刷盘周期
    flush_interval: Duration,
    // 会话判定阈值
    session_gap: Duration,
    // 是否暂停采集
    paused: bool,
    // 键盘监听是否正常工作
    keyboard_active: bool,
    // 是否忽略组合键（ctrl/alt/shift/cmd/fn + 其他键）
    ignore_key_combos: bool,
    // 菜单栏显示模式
    menu_bar_display_mode: MenuBarDisplayMode,
    // 最近一次错误信息（用于前端提示）
    last_error: Option<String>,
    // CSV 汇总文件路径
    pub log_path: PathBuf,
    // 应用运行日志文件路径
    pub app_log_path: PathBuf,
    // 明细数据的存储实现
    storage: Box<dyn DetailStorage>,
    #[cfg(not(target_os = "macos"))]
    modifier_state: ModifierState,
}

#[cfg(not(target_os = "macos"))]
#[derive(Default)]
struct ModifierState {
    ctrl: bool,
    alt: bool,
    shift: bool,
    meta: bool,
    function: bool,
}

#[cfg(not(target_os = "macos"))]
impl ModifierState {
    fn has_any_modifier(&self) -> bool {
        self.ctrl || self.alt || self.shift || self.meta || self.function
    }

    fn is_modifier_key(key: rdev::Key) -> bool {
        matches!(
            key,
            rdev::Key::ControlLeft
                | rdev::Key::ControlRight
                | rdev::Key::Alt
                | rdev::Key::AltGr
                | rdev::Key::ShiftLeft
                | rdev::Key::ShiftRight
                | rdev::Key::MetaLeft
                | rdev::Key::MetaRight
                | rdev::Key::Function
        )
    }

    fn update(&mut self, key: rdev::Key, pressed: bool) {
        match key {
            rdev::Key::ControlLeft | rdev::Key::ControlRight => self.ctrl = pressed,
            rdev::Key::Alt | rdev::Key::AltGr => self.alt = pressed,
            rdev::Key::ShiftLeft | rdev::Key::ShiftRight => self.shift = pressed,
            rdev::Key::MetaLeft | rdev::Key::MetaRight => self.meta = pressed,
            rdev::Key::Function => self.function = pressed,
            _ => {}
        }
    }
}

pub fn new_collector_state(
    log_path: PathBuf,
    app_log_path: PathBuf,
    detail_path: PathBuf,
    config: &AppConfig,
) -> CollectorState {
    let now = Instant::now();
    let storage: Box<dyn DetailStorage> = Box::new(JsonFileStorage { path: detail_path });
    let stats = storage.load_stats().unwrap_or_default();
    if !stats.is_empty() {
        let _ = append_app_log(
            &app_log_path,
            &format!("loaded {} detail rows from storage", stats.len()),
        );
    }
    CollectorState {
        stats,
        last_key_instant: now,
        last_flush_instant: now,
        collector_tick_interval: config.collector_tick_interval(),
        flush_interval: config.flush_interval(),
        session_gap: config.session_gap(),
        paused: false,
        keyboard_active: true,
        ignore_key_combos: config.ignore_key_combos,
        menu_bar_display_mode: config.menu_bar_display_mode,
        last_error: None,
        log_path,
        app_log_path,
        storage,
        #[cfg(not(target_os = "macos"))]
        modifier_state: ModifierState::default(),
    }
}

pub fn start_collector(state: Arc<Mutex<CollectorState>>) {
    let key_state = state.clone();
    let error_state = state.clone();
    std::thread::spawn(move || {
        #[cfg(target_os = "macos")]
        let result = listen_keypress_macos(key_state);

        #[cfg(not(target_os = "macos"))]
        let result = rdev::listen(move |event| match event.event_type {
            rdev::EventType::KeyPress(key) => on_key_event_non_macos(&key_state, key, true),
            rdev::EventType::KeyRelease(key) => on_key_event_non_macos(&key_state, key, false),
            _ => {}
        })
        .map_err(|e| format!("{:?}", e));

        if let Err(err) = result {
            if let Ok(mut locked) = error_state.lock() {
                locked.keyboard_active = false;
                locked.last_error = Some(err.to_string());
                let _ = append_app_log(
                    &locked.app_log_path,
                    &format!("keyboard listener error: {}", err),
                );
            }
        }
    });

    let tick_state = state;
    std::thread::spawn(move || loop {
        let tick_interval = if let Ok(locked) = tick_state.lock() {
            locked.collector_tick_interval
        } else {
            Duration::from_secs(1)
        };
        std::thread::sleep(tick_interval);
        if let Ok(mut locked) = tick_state.lock() {
            let now = Instant::now();
            if locked.paused {
                continue;
            }
            if now.duration_since(locked.last_flush_instant) >= locked.flush_interval {
                locked.last_flush_instant = now;
                let _ = locked.storage.save_stats(&locked.stats);
                if let Ok(rows) = snapshot_rows(&locked) {
                    let _ = write_csv(&locked.log_path, &rows);
                }
            }
        }
    });
}

fn on_key_press(state: &Arc<Mutex<CollectorState>>, is_key_combo: bool) {
    if let Ok(mut locked) = state.lock() {
        let now = Instant::now();
        if locked.paused {
            locked.last_key_instant = now;
            return;
        }
        if should_ignore_keypress(locked.ignore_key_combos, is_key_combo) {
            return;
        }
        let delta = now.duration_since(locked.last_key_instant);
        locked.last_key_instant = now;
        let (app_name, window_title) = active_window_info();
        let key = StatsKey {
            date: current_minute(),
            app_name,
            window_title,
        };
        let session_gap = locked.session_gap;
        let entry = locked.stats.entry(key).or_insert(StatsValue {
            active_typing_ms: 0,
            key_count: 0,
            session_count: 0,
        });
        entry.key_count += 1;
        if delta <= session_gap {
            entry.active_typing_ms += delta.as_millis() as u64;
        } else {
            entry.session_count += 1;
        }
    }
}

fn should_ignore_keypress(ignore_key_combos: bool, is_key_combo: bool) -> bool {
    ignore_key_combos && is_key_combo
}

#[cfg(not(target_os = "macos"))]
fn on_key_event_non_macos(state: &Arc<Mutex<CollectorState>>, key: rdev::Key, pressed: bool) {
    let (is_modifier_key, has_modifier_before) = if let Ok(mut locked) = state.lock() {
        let is_modifier_key = ModifierState::is_modifier_key(key);
        let has_modifier_before = locked.modifier_state.has_any_modifier();
        locked.modifier_state.update(key, pressed);
        (is_modifier_key, has_modifier_before)
    } else {
        return;
    };

    if pressed {
        on_key_press(state, has_modifier_before && !is_modifier_key);
    }
}

#[cfg(target_os = "macos")]
fn listen_keypress_macos(state: Arc<Mutex<CollectorState>>) -> Result<(), String> {
    use std::ffi::c_void;

    type CFMachPortRef = *const c_void;
    type CFIndex = i64;
    type CFAllocatorRef = *const c_void;
    type CFRunLoopSourceRef = *const c_void;
    type CFRunLoopRef = *const c_void;
    type CFRunLoopMode = *const c_void;

    type CGEventTapProxy = *const c_void;
    type CGEventRef = *const c_void;
    type CGEventTapLocation = u32;
    type CGEventTapPlacement = u32;
    type CGEventTapOptions = u32;
    type CGEventMask = u64;
    type CGEventType = u32;

    const CG_EVENT_TAP_LOCATION_HID: CGEventTapLocation = 0;
    const CG_EVENT_TAP_PLACEMENT_HEAD_INSERT: CGEventTapPlacement = 0;
    const CG_EVENT_TAP_OPTION_LISTEN_ONLY: CGEventTapOptions = 1;
    const CG_EVENT_TYPE_KEY_DOWN: CGEventType = 10;
    type CGEventFlags = u64;
    const CG_EVENT_FLAG_MASK_SHIFT: CGEventFlags = 1 << 17;
    const CG_EVENT_FLAG_MASK_CONTROL: CGEventFlags = 1 << 18;
    const CG_EVENT_FLAG_MASK_ALTERNATE: CGEventFlags = 1 << 19;
    const CG_EVENT_FLAG_MASK_COMMAND: CGEventFlags = 1 << 20;
    const CG_EVENT_FLAG_MASK_SECONDARY_FN: CGEventFlags = 1 << 23;

    extern "C" {
        fn CGEventTapCreate(
            tap: CGEventTapLocation,
            place: CGEventTapPlacement,
            options: CGEventTapOptions,
            events_of_interest: CGEventMask,
            callback: unsafe extern "C" fn(
                CGEventTapProxy,
                CGEventType,
                CGEventRef,
                *mut c_void,
            ) -> CGEventRef,
            user_info: *mut c_void,
        ) -> CFMachPortRef;
        fn CFMachPortCreateRunLoopSource(
            allocator: CFAllocatorRef,
            port: CFMachPortRef,
            order: CFIndex,
        ) -> CFRunLoopSourceRef;
        fn CFRunLoopGetCurrent() -> CFRunLoopRef;
        fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
        fn CFRunLoopRun();
        fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
        fn CGEventGetFlags(event: CGEventRef) -> CGEventFlags;
        static kCFRunLoopCommonModes: CFRunLoopMode;
    }

    unsafe extern "C" fn callback(
        _proxy: CGEventTapProxy,
        type_: CGEventType,
        event: CGEventRef,
        user_info: *mut c_void,
    ) -> CGEventRef {
        if type_ == CG_EVENT_TYPE_KEY_DOWN {
            let state = &*(user_info as *const Arc<Mutex<CollectorState>>);
            let flags = CGEventGetFlags(event);
            let has_modifier = flags
                & (CG_EVENT_FLAG_MASK_SHIFT
                    | CG_EVENT_FLAG_MASK_CONTROL
                    | CG_EVENT_FLAG_MASK_ALTERNATE
                    | CG_EVENT_FLAG_MASK_COMMAND
                    | CG_EVENT_FLAG_MASK_SECONDARY_FN)
                != 0;
            on_key_press(state, has_modifier);
        }
        event
    }

    let user_info = Box::into_raw(Box::new(state)) as *mut c_void;
    unsafe {
        let tap = CGEventTapCreate(
            CG_EVENT_TAP_LOCATION_HID,
            CG_EVENT_TAP_PLACEMENT_HEAD_INSERT,
            CG_EVENT_TAP_OPTION_LISTEN_ONLY,
            1u64 << CG_EVENT_TYPE_KEY_DOWN,
            callback,
            user_info,
        );
        if tap.is_null() {
            return Err("EventTapCreate failed (need Accessibility permission?)".to_string());
        }
        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
        if source.is_null() {
            return Err("CFMachPortCreateRunLoopSource failed".to_string());
        }
        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
        CFRunLoopRun();
    }
    Ok(())
}

pub fn snapshot_rows(state: &CollectorState) -> Result<Vec<StatsRow>, String> {
    let mut rows: Vec<StatsRow> = state
        .stats
        .iter()
        .map(|(key, value)| StatsRow {
            date: key.date.clone(),
            app_name: key.app_name.clone(),
            window_title: key.window_title.clone(),
            active_typing_ms: value.active_typing_ms,
            key_count: value.key_count,
            session_count: value.session_count,
        })
        .collect();
    rows.sort_by(|a, b| {
        (&a.date, &a.app_name, &a.window_title, a.active_typing_ms).cmp(&(
            &b.date,
            &b.app_name,
            &b.window_title,
            b.active_typing_ms,
        ))
    });
    Ok(rows)
}

pub fn snapshot(state: &CollectorState) -> StatsSnapshot {
    let rows = snapshot_rows(state).unwrap_or_default();
    StatsSnapshot {
        rows,
        paused: state.paused,
        keyboard_active: state.keyboard_active,
        ignore_key_combos: state.ignore_key_combos,
        tray_display_mode: state.menu_bar_display_mode.as_str().to_string(),
        last_error: state.last_error.clone(),
        log_path: state.log_path.to_string_lossy().to_string(),
    }
}

pub fn set_paused(state: &mut CollectorState, paused: bool) {
    state.paused = paused;
}

pub fn set_ignore_key_combos(state: &mut CollectorState, ignore_key_combos: bool) {
    state.ignore_key_combos = ignore_key_combos;
}

pub fn set_menu_bar_display_mode(state: &mut CollectorState, mode: MenuBarDisplayMode) {
    state.menu_bar_display_mode = mode;
}

pub fn clear_stats(state: &mut CollectorState) {
    state.stats.clear();
    let _ = state.storage.save_stats(&state.stats);
}

fn current_minute() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        set_ignore_key_combos, should_ignore_keypress, snapshot, snapshot_rows, CollectorState,
        StatsKey, StatsValue,
    };
    use crate::app_config::MenuBarDisplayMode;
    use crate::storage::JsonFileStorage;
    use std::{
        collections::HashMap,
        path::PathBuf,
        time::{Duration, Instant},
    };

    fn build_state(stats: HashMap<StatsKey, StatsValue>) -> CollectorState {
        let now = Instant::now();
        CollectorState {
            stats,
            last_key_instant: now,
            last_flush_instant: now,
            collector_tick_interval: Duration::from_secs(1),
            flush_interval: Duration::from_secs(60),
            session_gap: Duration::from_secs(5),
            paused: false,
            keyboard_active: true,
            ignore_key_combos: false,
            menu_bar_display_mode: MenuBarDisplayMode::IconText,
            last_error: None,
            log_path: PathBuf::from("log.csv"),
            app_log_path: PathBuf::from("app.log"),
            storage: Box::new(JsonFileStorage {
                path: PathBuf::from("detail.json"),
            }),
            #[cfg(not(target_os = "macos"))]
            modifier_state: ModifierState::default(),
        }
    }

    #[test]
    fn snapshot_rows_sorted_by_keys() {
        let mut stats = HashMap::new();
        stats.insert(
            StatsKey {
                date: "2026-02-09 10:01".to_string(),
                app_name: "B".to_string(),
                window_title: "TitleB".to_string(),
            },
            StatsValue {
                active_typing_ms: 500,
                key_count: 5,
                session_count: 1,
            },
        );
        stats.insert(
            StatsKey {
                date: "2026-02-09 10:00".to_string(),
                app_name: "A".to_string(),
                window_title: "TitleA".to_string(),
            },
            StatsValue {
                active_typing_ms: 800,
                key_count: 8,
                session_count: 2,
            },
        );
        let state = build_state(stats);
        let rows = snapshot_rows(&state).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].date, "2026-02-09 10:00");
        assert_eq!(rows[0].app_name, "A");
        assert_eq!(rows[1].date, "2026-02-09 10:01");
        assert_eq!(rows[1].app_name, "B");
    }

    #[test]
    fn should_ignore_keypress_only_when_both_enabled_and_combo() {
        assert!(!should_ignore_keypress(false, false));
        assert!(!should_ignore_keypress(false, true));
        assert!(!should_ignore_keypress(true, false));
        assert!(should_ignore_keypress(true, true));
    }

    #[test]
    fn set_ignore_key_combos_reflects_in_snapshot() {
        let state = build_state(HashMap::new());
        let mut state = state;
        assert!(!snapshot(&state).ignore_key_combos);
        set_ignore_key_combos(&mut state, true);
        assert!(snapshot(&state).ignore_key_combos);
        set_ignore_key_combos(&mut state, false);
        assert!(!snapshot(&state).ignore_key_combos);
    }
}

fn active_window_info() -> (String, String) {
    if let Ok(window) = active_win_pos_rs::get_active_window() {
        let app = window.app_name;
        let title = window.title;
        #[cfg(target_os = "macos")]
        {
            if let Some(bundle_id) = bundle_id_from_path(&window.process_path) {
                return (bundle_id, title);
            }
        }
        return (app, title);
    }
    ("Unknown".to_string(), "".to_string())
}

#[cfg(target_os = "macos")]
fn bundle_id_from_path(path: &std::path::Path) -> Option<String> {
    let mut bundle_path: Option<PathBuf> = None;
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if let Some(name) = current.file_name().and_then(|v| v.to_str()) {
            if name.ends_with(".app") {
                bundle_path = Some(current.clone());
            }
        }
    }
    let bundle_path = bundle_path?;
    let info_plist = bundle_path.join("Contents").join("Info.plist");
    let value = plist::Value::from_file(info_plist).ok()?;
    match value.as_dictionary() {
        Some(dict) => dict
            .get("CFBundleIdentifier")
            .and_then(|v| v.as_string())
            .map(|v| v.to_string()),
        None => None,
    }
}

fn write_csv(path: &PathBuf, rows: &[StatsRow]) -> Result<(), String> {
    let mut file = File::create(path).map_err(|e| e.to_string())?;
    writeln!(
        file,
        "date,app_name,window_title,active_typing_ms,key_count,session_count"
    )
    .map_err(|e| e.to_string())?;
    for row in rows {
        let line = format!(
            "{},{},{},{},{},{}",
            escape_csv(&row.date),
            escape_csv(&row.app_name),
            escape_csv(&row.window_title),
            row.active_typing_ms,
            row.key_count,
            row.session_count
        );
        writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        let escaped = value.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        value.to_string()
    }
}

pub fn append_app_log(path: &PathBuf, message: &str) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    let line = format!("{} {}", Local::now().format("%Y-%m-%d %H:%M:%S"), message);
    writeln!(file, "{}", line).map_err(|e| e.to_string())?;
    print_line_if_dev(&line);
    Ok(())
}

fn print_line_if_dev(line: &str) {
    if cfg!(debug_assertions) {
        println!("[TypePulse] {}", line);
    }
}
