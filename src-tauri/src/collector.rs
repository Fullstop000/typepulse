use std::{
    collections::{HashMap, HashSet},
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
    pub auto_paused: bool,
    pub auto_pause_reason: Option<String>,
    pub keyboard_active: bool,
    pub ignore_key_combos: bool,
    pub excluded_bundle_ids: Vec<String>,
    pub one_password_suggestion_pending: bool,
    pub tray_display_mode: String,
    pub last_error: Option<String>,
    pub log_path: String,
}

pub struct CollectorState {
    // 统计聚合的明细数据（按时间/应用/窗口维度）
    stats: HashMap<StatsKey, StatsValue>,
    // 最近一次“有效输入活动”时间点，用于计算会话间隔
    last_typing_instant: Instant,
    // 最近一次 tick 时间点，用于精确累加 active_typing_ms
    last_tick_instant: Instant,
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
    // 当前是否因黑名单/安全输入而自动暂停记录
    auto_paused: bool,
    // 自动暂停原因（blacklist/secure_input）
    auto_pause_reason: Option<String>,
    // 键盘监听是否正常工作
    keyboard_active: bool,
    // 是否忽略组合键（ctrl/alt/shift/cmd/fn + 其他键）
    ignore_key_combos: bool,
    // 菜单栏显示模式
    menu_bar_display_mode: MenuBarDisplayMode,
    // 忽略采集应用的 Bundle ID 列表
    excluded_bundle_ids: HashSet<String>,
    // 首次 1Password 建议是否待处理
    one_password_suggestion_pending: bool,
    // 最近一次错误信息（用于前端提示）
    last_error: Option<String>,
    // 当前按下的非修饰键集合（用于消除长按自动重复）
    pressed_non_modifier_keys: HashSet<String>,
    // 当前持续输入归属的统计维度键（用于 tick 累加 active_typing_ms）
    active_stats_key: Option<StatsKey>,
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
        last_typing_instant: now,
        last_tick_instant: now,
        last_flush_instant: now,
        collector_tick_interval: config.collector_tick_interval(),
        flush_interval: config.flush_interval(),
        session_gap: config.session_gap(),
        paused: false,
        auto_paused: false,
        auto_pause_reason: None,
        keyboard_active: true,
        ignore_key_combos: config.ignore_key_combos,
        menu_bar_display_mode: config.menu_bar_display_mode,
        excluded_bundle_ids: config
            .excluded_bundle_ids
            .iter()
            .map(|v| v.to_ascii_lowercase())
            .collect(),
        one_password_suggestion_pending: false,
        last_error: None,
        pressed_non_modifier_keys: HashSet::new(),
        active_stats_key: None,
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
            let elapsed = now.duration_since(locked.last_tick_instant);
            locked.last_tick_instant = now;
            apply_collector_event(
                &mut locked,
                CollectorEvent::Tick {
                    elapsed,
                    capture_context: capture_context(),
                    at: now,
                },
            );
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

// Reset runtime key states when capture is paused to avoid stale key-down state.
fn reset_active_typing_state(state: &mut CollectorState) {
    state.pressed_non_modifier_keys.clear();
    state.active_stats_key = None;
    #[cfg(not(target_os = "macos"))]
    {
        state.modifier_state = ModifierState::default();
    }
}

// Build the current aggregation key from capture context.
fn stats_key_from_context(capture_context: &CaptureContext) -> StatsKey {
    let app_name = capture_context
        .bundle_id
        .clone()
        .unwrap_or_else(|| capture_context.app_name.clone());
    StatsKey {
        date: current_minute(),
        app_name,
        window_title: capture_context.window_title.clone(),
    }
}

// Apply a non-modifier key-down event. Repeated key-down of the same physical key is ignored.
fn apply_non_modifier_key_down(
    state: &mut CollectorState,
    key_id: String,
    is_key_combo: bool,
    capture_context: CaptureContext,
    now: Instant,
) {
    state.auto_paused = is_auto_paused(state, &capture_context);
    state.auto_pause_reason = auto_pause_reason(state, &capture_context);
    if state.paused
        || state.auto_paused
        || should_ignore_keypress(state.ignore_key_combos, is_key_combo)
    {
        return;
    }
    if !state.pressed_non_modifier_keys.insert(key_id) {
        return;
    }
    let key = stats_key_from_context(&capture_context);
    let delta = now.duration_since(state.last_typing_instant);
    let session_gap = state.session_gap;
    let entry = state.stats.entry(key.clone()).or_insert(StatsValue {
        active_typing_ms: 0,
        key_count: 0,
        session_count: 0,
    });
    entry.key_count += 1;
    if delta > session_gap {
        entry.session_count += 1;
    }
    state.last_typing_instant = now;
    state.active_stats_key = Some(key);
}

// Apply a non-modifier key-up event and clear active typing key when all keys are released.
fn apply_non_modifier_key_up(state: &mut CollectorState, key_id: &str) {
    state.pressed_non_modifier_keys.remove(key_id);
    if state.pressed_non_modifier_keys.is_empty() {
        state.active_stats_key = None;
    }
}

// Accumulate active typing time from wall-clock tick while there is at least one key held down.
fn accumulate_active_typing_for_tick(state: &mut CollectorState, elapsed: Duration, now: Instant) {
    if state.pressed_non_modifier_keys.is_empty() {
        return;
    }
    let Some(key) = state.active_stats_key.clone() else {
        return;
    };
    let entry = state.stats.entry(key).or_insert(StatsValue {
        active_typing_ms: 0,
        key_count: 0,
        session_count: 0,
    });
    entry.active_typing_ms += elapsed.as_millis() as u64;
    state.last_typing_instant = now;
}

fn on_non_modifier_key_down(
    state: &Arc<Mutex<CollectorState>>,
    key_id: String,
    is_key_combo: bool,
) {
    if let Ok(mut locked) = state.lock() {
        apply_collector_event(
            &mut locked,
            CollectorEvent::NonModifierKeyDown {
                key_id,
                is_key_combo,
                capture_context: capture_context(),
                at: Instant::now(),
            },
        );
    }
}

fn on_non_modifier_key_up(state: &Arc<Mutex<CollectorState>>, key_id: &str) {
    if let Ok(mut locked) = state.lock() {
        apply_collector_event(
            &mut locked,
            CollectorEvent::NonModifierKeyUp {
                key_id: key_id.to_string(),
            },
        );
    }
}

// Apply one collector event to state. This keeps runtime and test event semantics aligned.
fn apply_collector_event(state: &mut CollectorState, event: CollectorEvent) {
    match event {
        CollectorEvent::NonModifierKeyDown {
            key_id,
            is_key_combo,
            capture_context,
            at,
        } => apply_non_modifier_key_down(state, key_id, is_key_combo, capture_context, at),
        CollectorEvent::NonModifierKeyUp { key_id } => apply_non_modifier_key_up(state, &key_id),
        CollectorEvent::Tick {
            elapsed,
            capture_context,
            at,
        } => {
            state.auto_paused = is_auto_paused(state, &capture_context);
            state.auto_pause_reason = auto_pause_reason(state, &capture_context);
            if state.paused || state.auto_paused {
                reset_active_typing_state(state);
                return;
            }
            accumulate_active_typing_for_tick(state, elapsed, at);
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

    if is_modifier_key {
        return;
    }

    let key_id = format!("rdev:{:?}", key);
    if pressed {
        on_non_modifier_key_down(state, key_id, has_modifier_before);
    } else {
        on_non_modifier_key_up(state, &key_id);
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
    const CG_EVENT_TYPE_KEY_UP: CGEventType = 11;
    type CGEventFlags = u64;
    const CG_EVENT_FLAG_MASK_SHIFT: CGEventFlags = 1 << 17;
    const CG_EVENT_FLAG_MASK_CONTROL: CGEventFlags = 1 << 18;
    const CG_EVENT_FLAG_MASK_ALTERNATE: CGEventFlags = 1 << 19;
    const CG_EVENT_FLAG_MASK_COMMAND: CGEventFlags = 1 << 20;
    const CG_EVENT_FLAG_MASK_SECONDARY_FN: CGEventFlags = 1 << 23;
    type CGEventField = u32;
    const CG_EVENT_FIELD_KEYBOARD_EVENT_KEYCODE: CGEventField = 9;

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
        fn CGEventGetIntegerValueField(event: CGEventRef, field: CGEventField) -> i64;
        static kCFRunLoopCommonModes: CFRunLoopMode;
    }

    unsafe extern "C" fn callback(
        _proxy: CGEventTapProxy,
        type_: CGEventType,
        event: CGEventRef,
        user_info: *mut c_void,
    ) -> CGEventRef {
        if type_ == CG_EVENT_TYPE_KEY_DOWN || type_ == CG_EVENT_TYPE_KEY_UP {
            let state = &*(user_info as *const Arc<Mutex<CollectorState>>);
            let flags = CGEventGetFlags(event);
            let key_code =
                CGEventGetIntegerValueField(event, CG_EVENT_FIELD_KEYBOARD_EVENT_KEYCODE);
            let key_id = format!("mac:{}", key_code);
            let has_modifier = flags
                & (CG_EVENT_FLAG_MASK_SHIFT
                    | CG_EVENT_FLAG_MASK_CONTROL
                    | CG_EVENT_FLAG_MASK_ALTERNATE
                    | CG_EVENT_FLAG_MASK_COMMAND
                    | CG_EVENT_FLAG_MASK_SECONDARY_FN)
                != 0;
            if type_ == CG_EVENT_TYPE_KEY_DOWN {
                on_non_modifier_key_down(state, key_id, has_modifier);
            } else {
                on_non_modifier_key_up(state, &key_id);
            }
        }
        event
    }

    let user_info = Box::into_raw(Box::new(state)) as *mut c_void;
    unsafe {
        let tap = CGEventTapCreate(
            CG_EVENT_TAP_LOCATION_HID,
            CG_EVENT_TAP_PLACEMENT_HEAD_INSERT,
            CG_EVENT_TAP_OPTION_LISTEN_ONLY,
            (1u64 << CG_EVENT_TYPE_KEY_DOWN) | (1u64 << CG_EVENT_TYPE_KEY_UP),
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
    let mut excluded_bundle_ids: Vec<String> = state.excluded_bundle_ids.iter().cloned().collect();
    excluded_bundle_ids.sort();
    StatsSnapshot {
        rows,
        paused: state.paused,
        auto_paused: state.auto_paused,
        auto_pause_reason: state.auto_pause_reason.clone(),
        keyboard_active: state.keyboard_active,
        ignore_key_combos: state.ignore_key_combos,
        excluded_bundle_ids,
        one_password_suggestion_pending: state.one_password_suggestion_pending,
        tray_display_mode: state.menu_bar_display_mode.as_str().to_string(),
        last_error: state.last_error.clone(),
        log_path: state.log_path.to_string_lossy().to_string(),
    }
}

pub fn set_paused(state: &mut CollectorState, paused: bool) {
    state.paused = paused;
    if paused {
        reset_active_typing_state(state);
    }
}

pub fn set_ignore_key_combos(state: &mut CollectorState, ignore_key_combos: bool) {
    state.ignore_key_combos = ignore_key_combos;
}

pub fn set_menu_bar_display_mode(state: &mut CollectorState, mode: MenuBarDisplayMode) {
    state.menu_bar_display_mode = mode;
}

pub fn set_excluded_bundle_ids(state: &mut CollectorState, bundle_ids: &[String]) {
    state.excluded_bundle_ids = bundle_ids
        .iter()
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
        .collect();
}

pub fn add_excluded_bundle_id(state: &mut CollectorState, bundle_id: &str) -> bool {
    let normalized = bundle_id.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }
    state.excluded_bundle_ids.insert(normalized)
}

pub fn remove_excluded_bundle_id(state: &mut CollectorState, bundle_id: &str) -> bool {
    state
        .excluded_bundle_ids
        .remove(&bundle_id.trim().to_ascii_lowercase())
}

pub fn set_one_password_suggestion_pending(state: &mut CollectorState, pending: bool) {
    state.one_password_suggestion_pending = pending;
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
        apply_collector_event, set_ignore_key_combos, should_ignore_keypress, snapshot,
        snapshot_rows, CaptureContext, CollectorEvent, CollectorState, StatsKey, StatsValue,
    };
    use crate::app_config::MenuBarDisplayMode;
    use crate::storage::JsonFileStorage;
    use std::{
        collections::{HashMap, HashSet},
        path::PathBuf,
        time::{Duration, Instant},
    };

    fn build_state(stats: HashMap<StatsKey, StatsValue>) -> CollectorState {
        let now = Instant::now();
        CollectorState {
            stats,
            last_typing_instant: now,
            last_tick_instant: now,
            last_flush_instant: now,
            collector_tick_interval: Duration::from_secs(1),
            flush_interval: Duration::from_secs(60),
            session_gap: Duration::from_secs(5),
            paused: false,
            auto_paused: false,
            auto_pause_reason: None,
            keyboard_active: true,
            ignore_key_combos: false,
            menu_bar_display_mode: MenuBarDisplayMode::IconText,
            excluded_bundle_ids: HashSet::new(),
            one_password_suggestion_pending: false,
            last_error: None,
            pressed_non_modifier_keys: HashSet::new(),
            active_stats_key: None,
            log_path: PathBuf::from("log.csv"),
            app_log_path: PathBuf::from("app.log"),
            storage: Box::new(JsonFileStorage {
                path: PathBuf::from("detail.json"),
            }),
            #[cfg(not(target_os = "macos"))]
            modifier_state: ModifierState::default(),
        }
    }

    // Event-stream harness for collector unit tests. Tests can feed key/tick events in order.
    struct CollectorEventHarness {
        state: CollectorState,
        default_context: CaptureContext,
    }

    impl CollectorEventHarness {
        fn new() -> Self {
            Self {
                state: build_state(HashMap::new()),
                default_context: CaptureContext {
                    app_name: "Editor".to_string(),
                    window_title: "Doc".to_string(),
                    bundle_id: Some("com.test.editor".to_string()),
                    secure_input: false,
                },
            }
        }

        // Push one synthetic collector event.
        fn push(&mut self, event: CollectorEvent) {
            apply_collector_event(&mut self.state, event);
        }

        // Push key-down with default capture context.
        fn key_down(&mut self, key_id: &str, is_key_combo: bool, at: Instant) {
            self.push(CollectorEvent::NonModifierKeyDown {
                key_id: key_id.to_string(),
                is_key_combo,
                capture_context: self.default_context.clone(),
                at,
            });
        }

        // Push key-up for one key id.
        fn key_up(&mut self, key_id: &str) {
            self.push(CollectorEvent::NonModifierKeyUp {
                key_id: key_id.to_string(),
            });
        }

        // Push one tick event with synthetic elapsed time.
        fn tick(&mut self, elapsed: Duration, at: Instant) {
            self.push(CollectorEvent::Tick {
                elapsed,
                capture_context: self.default_context.clone(),
                at,
            });
        }

        fn rows(&self) -> Vec<super::StatsRow> {
            snapshot_rows(&self.state).unwrap()
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

    #[test]
    fn repeated_key_down_is_counted_once_until_key_up() {
        let mut harness = CollectorEventHarness::new();
        let now = Instant::now();

        harness.key_down("k:a", false, now);
        harness.key_down("k:a", false, now + Duration::from_millis(100));

        let rows = harness.rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].key_count, 1);

        harness.key_up("k:a");
        harness.key_down("k:a", false, now + Duration::from_millis(200));

        let rows = harness.rows();
        assert_eq!(rows[0].key_count, 2);
    }

    #[test]
    fn active_typing_ms_is_accumulated_from_tick_while_key_held() {
        let mut harness = CollectorEventHarness::new();
        let now = Instant::now();

        harness.key_down("k:a", false, now);
        harness.tick(
            Duration::from_millis(1200),
            now + Duration::from_millis(1200),
        );

        let rows = harness.rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].active_typing_ms, 1200);
    }

    #[test]
    fn event_stream_stops_accumulating_after_key_up() {
        let mut harness = CollectorEventHarness::new();
        let now = Instant::now();

        harness.key_down("k:a", false, now);
        harness.tick(Duration::from_millis(500), now + Duration::from_millis(500));
        harness.key_up("k:a");
        harness.tick(
            Duration::from_millis(700),
            now + Duration::from_millis(1200),
        );

        let rows = harness.rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].active_typing_ms, 500);
        assert_eq!(rows[0].key_count, 1);
    }
}

#[derive(Serialize, Clone)]
pub struct RunningAppInfo {
    pub bundle_id: String,
    pub name: String,
}

#[derive(Clone)]
struct CaptureContext {
    app_name: String,
    window_title: String,
    bundle_id: Option<String>,
    secure_input: bool,
}

// Unified collector event model used by runtime handlers and unit tests.
#[derive(Clone)]
enum CollectorEvent {
    NonModifierKeyDown {
        key_id: String,
        is_key_combo: bool,
        capture_context: CaptureContext,
        at: Instant,
    },
    NonModifierKeyUp {
        key_id: String,
    },
    Tick {
        elapsed: Duration,
        capture_context: CaptureContext,
        at: Instant,
    },
}

fn capture_context() -> CaptureContext {
    let secure_input = is_secure_event_input_enabled();
    if let Ok(window) = active_win_pos_rs::get_active_window() {
        let app_name = window.app_name;
        let window_title = window.title;
        #[cfg(target_os = "macos")]
        {
            let bundle_id = frontmost_bundle_id_macos()
                .or_else(|| bundle_id_from_path(&window.process_path))
                .map(|v| v.to_ascii_lowercase());
            return CaptureContext {
                app_name,
                window_title,
                bundle_id,
                secure_input,
            };
        }
        #[cfg(not(target_os = "macos"))]
        {
            return CaptureContext {
                app_name,
                window_title,
                bundle_id: None,
                secure_input,
            };
        }
    }
    CaptureContext {
        app_name: "Unknown".to_string(),
        window_title: String::new(),
        bundle_id: None,
        secure_input,
    }
}

fn is_auto_paused(state: &CollectorState, context: &CaptureContext) -> bool {
    is_excluded_app(state, context) || context.secure_input
}

fn auto_pause_reason(state: &CollectorState, context: &CaptureContext) -> Option<String> {
    if context.secure_input {
        return Some("secure_input".to_string());
    }
    if is_excluded_app(state, context) {
        return Some("blacklist".to_string());
    }
    None
}

fn is_excluded_app(state: &CollectorState, context: &CaptureContext) -> bool {
    match &context.bundle_id {
        Some(bundle_id) => state
            .excluded_bundle_ids
            .contains(&bundle_id.to_ascii_lowercase()),
        None => false,
    }
}

pub fn running_apps() -> Vec<RunningAppInfo> {
    #[cfg(target_os = "macos")]
    {
        return workspace_running_apps();
    }
    #[cfg(not(target_os = "macos"))]
    {
        vec![]
    }
}

pub fn bundle_id_from_app_path(path: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        bundle_id_from_path(std::path::Path::new(path)).map(|v| v.to_ascii_lowercase())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        None
    }
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

#[cfg(target_os = "macos")]
fn is_secure_event_input_enabled() -> bool {
    #[link(name = "Carbon", kind = "framework")]
    extern "C" {
        fn IsSecureEventInputEnabled() -> bool;
    }
    unsafe { IsSecureEventInputEnabled() }
}

#[cfg(not(target_os = "macos"))]
fn is_secure_event_input_enabled() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn frontmost_bundle_id_macos() -> Option<String> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace == nil {
            return None;
        }
        let app: id = msg_send![workspace, frontmostApplication];
        if app == nil {
            return None;
        }
        let bundle_id: id = msg_send![app, bundleIdentifier];
        if bundle_id == nil {
            return None;
        }
        Some(nsstring_to_string(bundle_id))
    }
}

#[cfg(target_os = "macos")]
fn workspace_running_apps() -> Vec<RunningAppInfo> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};
    use std::collections::BTreeMap;

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace == nil {
            return vec![];
        }
        let apps: id = msg_send![workspace, runningApplications];
        if apps == nil {
            return vec![];
        }
        let count: usize = msg_send![apps, count];
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        for idx in 0..count {
            let app: id = msg_send![apps, objectAtIndex: idx];
            if app == nil {
                continue;
            }
            let bundle_id_obj: id = msg_send![app, bundleIdentifier];
            if bundle_id_obj == nil {
                continue;
            }
            let name_obj: id = msg_send![app, localizedName];
            let bundle_id = nsstring_to_string(bundle_id_obj);
            let name = if name_obj == nil {
                bundle_id.clone()
            } else {
                nsstring_to_string(name_obj)
            };
            if !bundle_id.trim().is_empty() {
                map.insert(bundle_id.to_ascii_lowercase(), name);
            }
        }
        map.into_iter()
            .map(|(bundle_id, name)| RunningAppInfo { bundle_id, name })
            .collect()
    }
}

#[cfg(target_os = "macos")]
fn nsstring_to_string(value: cocoa::base::id) -> String {
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let bytes: *const std::os::raw::c_char = msg_send![value, UTF8String];
        if bytes.is_null() {
            return String::new();
        }
        std::ffi::CStr::from_ptr(bytes)
            .to_string_lossy()
            .to_string()
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
