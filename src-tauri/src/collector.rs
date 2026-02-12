use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chrono::{Duration as ChronoDuration, Local, TimeZone};
use serde::Serialize;

use crate::app_config::{AppConfig, MenuBarDisplayMode};
use crate::storage::{
    DetailStorage, JsonFileStorage, StoredInputAnalytics, StoredInputEventChunk,
    StoredShortcutUsage,
};

const INPUT_CHUNK_WINDOW_MS: i64 = 5_000;
const INPUT_CHUNK_MAX_EVENTS: usize = 500;
const INPUT_CHUNK_MAX_STORED: usize = 20_000;

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

/// App-level usage count for one shortcut in snapshot payload.
#[derive(Serialize, Clone)]
pub struct ShortcutAppUsageRow {
    pub app_name: String,
    pub count: u64,
}

/// Shortcut leaderboard row used by frontend rendering.
#[derive(Serialize, Clone)]
pub struct ShortcutStatRow {
    pub shortcut_id: String,
    pub count: u64,
    pub apps: Vec<ShortcutAppUsageRow>,
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
    pub shortcut_stats: Vec<ShortcutStatRow>,
}

/// Runtime aggregate for one normalized shortcut id.
#[derive(Clone, Default)]
pub(crate) struct ShortcutUsageValue {
    pub(crate) count: u64,
    pub(crate) by_app: HashMap<String, u64>,
}

/// Persistable input chunk that stores compact event strings `dt,t,k,m`.
#[derive(Clone)]
struct InputEventChunk {
    v: u8,
    chunk_start_ms: i64,
    app_ref: u32,
    events: Vec<String>,
}

/// Open chunk that still accepts incoming events before it is rotated/flushed.
#[derive(Clone)]
struct OpenInputEventChunk {
    chunk_start_ms: i64,
    app_ref: u32,
    events: Vec<String>,
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
    // 快捷键聚合统计（key 为标准化 shortcut id）
    shortcut_usage: HashMap<String, ShortcutUsageValue>,
    // 应用字典（app_ref -> app_id），用于压缩事件 chunk 存储。
    app_dict: HashMap<u32, String>,
    // 反向应用字典（app_id -> app_ref），用于快速写入事件 chunk。
    app_ref_by_app: HashMap<String, u32>,
    // 下一个可用 app_ref 编号。
    next_app_ref: u32,
    // 已完成的事件 chunk（用于可选重算/调试）。
    event_chunks: Vec<InputEventChunk>,
    // 当前正在写入的 chunk。
    open_event_chunk: Option<OpenInputEventChunk>,
    // 快捷键规则：是否必须包含 Cmd/Ctrl。
    shortcut_require_cmd_or_ctrl: bool,
    // 快捷键规则：是否允许仅 Alt/Opt 作为主修饰键。
    shortcut_allow_alt_only: bool,
    // 快捷键规则：最小修饰键数量。
    shortcut_min_modifiers: u8,
    // 快捷键白名单（非空时，仅统计白名单内 shortcut id）。
    shortcut_allowlist: HashSet<String>,
    // 快捷键黑名单（优先级高于白名单）。
    shortcut_blocklist: HashSet<String>,
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

    fn snapshot(&self) -> ModifierSnapshot {
        ModifierSnapshot {
            ctrl: self.ctrl,
            opt: self.alt,
            shift: self.shift,
            cmd: self.meta,
            function: self.function,
        }
    }
}

/// Modifier snapshot used by shortcut normalization and event serialization.
#[derive(Clone, Copy, Default)]
struct ModifierSnapshot {
    ctrl: bool,
    opt: bool,
    shift: bool,
    cmd: bool,
    function: bool,
}

impl ModifierSnapshot {
    fn has_any(&self) -> bool {
        self.ctrl || self.opt || self.shift || self.cmd || self.function
    }

    fn has_shortcut_modifier(&self) -> bool {
        self.ctrl || self.cmd
    }

    fn bitmask(&self) -> u8 {
        (self.ctrl as u8)
            | ((self.opt as u8) << 1)
            | ((self.shift as u8) << 2)
            | ((self.cmd as u8) << 3)
            | ((self.function as u8) << 4)
    }

    // Rebuild modifier snapshot from persisted bitmask in compact input events.
    fn from_bitmask(mask: u8) -> Self {
        Self {
            ctrl: (mask & 0b00001) != 0,
            opt: (mask & 0b00010) != 0,
            shift: (mask & 0b00100) != 0,
            cmd: (mask & 0b01000) != 0,
            function: (mask & 0b10000) != 0,
        }
    }

    fn modifier_count(&self) -> u8 {
        self.ctrl as u8 + self.opt as u8 + self.shift as u8 + self.cmd as u8 + self.function as u8
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
    let analytics = storage.load_input_analytics().unwrap_or_default();
    let StoredInputAnalytics {
        shortcut_usage: stored_shortcut_usage,
        app_dict,
        next_app_ref,
        event_chunks: stored_event_chunks,
    } = analytics;
    let app_ref_by_app: HashMap<String, u32> = app_dict
        .iter()
        .map(|(app_ref, app_id)| (app_id.clone(), *app_ref))
        .collect();
    let shortcut_usage = stored_shortcut_usage
        .into_iter()
        .map(|(shortcut_id, usage)| {
            (
                shortcut_id,
                ShortcutUsageValue {
                    count: usage.count,
                    by_app: usage.by_app,
                },
            )
        })
        .collect();
    let event_chunks = stored_event_chunks
        .into_iter()
        .map(|chunk| InputEventChunk {
            v: chunk.v,
            chunk_start_ms: chunk.chunk_start_ms,
            app_ref: chunk.app_ref,
            events: chunk.events,
        })
        .collect();
    if !stats.is_empty() {
        let _ = append_app_log(
            &app_log_path,
            &format!("loaded {} detail rows from storage", stats.len()),
        );
    }
    let mut state = CollectorState {
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
        shortcut_usage,
        app_dict,
        app_ref_by_app,
        next_app_ref: next_app_ref.max(1),
        event_chunks,
        open_event_chunk: None,
        shortcut_require_cmd_or_ctrl: config.shortcut_require_cmd_or_ctrl,
        shortcut_allow_alt_only: config.shortcut_allow_alt_only,
        shortcut_min_modifiers: config.shortcut_min_modifiers.max(1),
        shortcut_allowlist: config
            .shortcut_allowlist
            .iter()
            .map(|v| v.to_ascii_lowercase())
            .collect(),
        shortcut_blocklist: config
            .shortcut_blocklist
            .iter()
            .map(|v| v.to_ascii_lowercase())
            .collect(),
        log_path,
        app_log_path,
        storage,
        #[cfg(not(target_os = "macos"))]
        modifier_state: ModifierState::default(),
    };
    // Rebuild shortcut aggregates when historical analytics only contains event chunks.
    if state.shortcut_usage.is_empty() && !state.event_chunks.is_empty() {
        rebuild_shortcut_usage_from_chunks(&mut state);
    }
    state
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
            flush_expired_open_chunk(&mut locked, current_ts_ms());
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
                let analytics = build_stored_input_analytics(&mut locked);
                let _ = locked.storage.save_input_analytics(&analytics);
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

// Build canonical shortcut id with deterministic modifier order:
// ctrl -> opt -> shift -> cmd -> key.
fn normalize_shortcut_id(modifiers: ModifierSnapshot, key: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if modifiers.ctrl {
        parts.push("ctrl");
    }
    if modifiers.opt {
        parts.push("opt");
    }
    if modifiers.shift {
        parts.push("shift");
    }
    if modifiers.cmd {
        parts.push("cmd");
    }
    parts.push(key);
    parts.join("_")
}

fn current_ts_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn app_id_from_context(capture_context: &CaptureContext) -> String {
    capture_context
        .bundle_id
        .clone()
        .unwrap_or_else(|| capture_context.app_name.clone())
}

// Resolve app_ref for the given app id and lazily register dictionary entry.
fn resolve_app_ref(state: &mut CollectorState, app_id: &str) -> u32 {
    if let Some(app_ref) = state.app_ref_by_app.get(app_id) {
        return *app_ref;
    }
    let app_ref = state.next_app_ref;
    state.next_app_ref = state.next_app_ref.saturating_add(1);
    state.app_ref_by_app.insert(app_id.to_string(), app_ref);
    state.app_dict.insert(app_ref, app_id.to_string());
    app_ref
}

fn push_finished_chunk(state: &mut CollectorState, chunk: OpenInputEventChunk) {
    if chunk.events.is_empty() {
        return;
    }
    state.event_chunks.push(InputEventChunk {
        v: 1,
        chunk_start_ms: chunk.chunk_start_ms,
        app_ref: chunk.app_ref,
        events: chunk.events,
    });
    if state.event_chunks.len() > INPUT_CHUNK_MAX_STORED {
        let overflow = state.event_chunks.len() - INPUT_CHUNK_MAX_STORED;
        state.event_chunks.drain(0..overflow);
    }
}

// Flush an open chunk when it is stale enough, reducing in-memory drift before periodic save.
fn flush_expired_open_chunk(state: &mut CollectorState, now_ms: i64) {
    let Some(open) = state.open_event_chunk.as_ref() else {
        return;
    };
    if now_ms - open.chunk_start_ms < INPUT_CHUNK_WINDOW_MS {
        return;
    }
    if let Some(chunk) = state.open_event_chunk.take() {
        push_finished_chunk(state, chunk);
    }
}

// Append compact input event string (`dt,t,k,m`) into 5s chunks grouped by app_ref.
fn append_input_event(
    state: &mut CollectorState,
    capture_context: &CaptureContext,
    event_type: char,
    key: &str,
    modifiers: ModifierSnapshot,
    now_ms: i64,
) {
    let app_ref = resolve_app_ref(state, &app_id_from_context(capture_context));
    let should_rotate = if let Some(open) = state.open_event_chunk.as_ref() {
        open.app_ref != app_ref
            || now_ms - open.chunk_start_ms >= INPUT_CHUNK_WINDOW_MS
            || open.events.len() >= INPUT_CHUNK_MAX_EVENTS
    } else {
        true
    };
    if should_rotate {
        if let Some(chunk) = state.open_event_chunk.take() {
            push_finished_chunk(state, chunk);
        }
        state.open_event_chunk = Some(OpenInputEventChunk {
            chunk_start_ms: now_ms,
            app_ref,
            events: Vec::new(),
        });
    }
    if let Some(open) = state.open_event_chunk.as_mut() {
        let dt = (now_ms - open.chunk_start_ms).max(0);
        open.events
            .push(format!("{dt},{event_type},{key},{}", modifiers.bitmask()));
    }
}

fn update_shortcut_usage(
    state: &mut CollectorState,
    capture_context: &CaptureContext,
    key: &str,
    modifiers: ModifierSnapshot,
) {
    let shortcut_id = normalize_shortcut_id(modifiers, key);
    if !should_count_shortcut(state, modifiers, &shortcut_id) {
        return;
    }
    let app_id = app_id_from_context(capture_context);
    let entry = state
        .shortcut_usage
        .entry(shortcut_id)
        .or_insert_with(ShortcutUsageValue::default);
    entry.count = entry.count.saturating_add(1);
    *entry.by_app.entry(app_id).or_insert(0) += 1;
}

// Centralized shortcut counting rule evaluator.
// Priority: blocklist > allowlist > baseline modifier rules.
fn should_count_shortcut(
    state: &CollectorState,
    modifiers: ModifierSnapshot,
    shortcut_id: &str,
) -> bool {
    if state.shortcut_blocklist.contains(shortcut_id) {
        return false;
    }
    if !state.shortcut_allowlist.is_empty() {
        return state.shortcut_allowlist.contains(shortcut_id);
    }
    if modifiers.modifier_count() < state.shortcut_min_modifiers {
        return false;
    }
    if state.shortcut_require_cmd_or_ctrl && !modifiers.has_shortcut_modifier() {
        return false;
    }
    if !state.shortcut_allow_alt_only
        && !modifiers.ctrl
        && !modifiers.cmd
        && modifiers.opt
        && !modifiers.shift
        && !modifiers.function
    {
        return false;
    }
    true
}

fn build_stored_input_analytics(state: &mut CollectorState) -> StoredInputAnalytics {
    if let Some(chunk) = state.open_event_chunk.take() {
        push_finished_chunk(state, chunk);
    }
    let shortcut_usage = state
        .shortcut_usage
        .iter()
        .map(|(shortcut_id, usage)| {
            (
                shortcut_id.clone(),
                StoredShortcutUsage {
                    count: usage.count,
                    by_app: usage.by_app.clone(),
                },
            )
        })
        .collect();
    let event_chunks = state
        .event_chunks
        .iter()
        .map(|chunk| StoredInputEventChunk {
            v: chunk.v,
            chunk_start_ms: chunk.chunk_start_ms,
            app_ref: chunk.app_ref,
            events: chunk.events.clone(),
        })
        .collect();
    StoredInputAnalytics {
        shortcut_usage,
        app_dict: state.app_dict.clone(),
        next_app_ref: state.next_app_ref,
        event_chunks,
    }
}

// Apply a non-modifier key-down event. Repeated key-down of the same physical key is ignored.
fn apply_non_modifier_key_down(
    state: &mut CollectorState,
    physical_key_id: String,
    shortcut_key: String,
    modifiers: ModifierSnapshot,
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
    if !state.pressed_non_modifier_keys.insert(physical_key_id) {
        return;
    }
    append_input_event(
        state,
        &capture_context,
        'd',
        &shortcut_key,
        modifiers,
        current_ts_ms(),
    );
    update_shortcut_usage(state, &capture_context, &shortcut_key, modifiers);
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
fn apply_non_modifier_key_up(
    state: &mut CollectorState,
    physical_key_id: &str,
    shortcut_key: &str,
    modifiers: ModifierSnapshot,
    capture_context: &CaptureContext,
) {
    append_input_event(
        state,
        capture_context,
        'u',
        shortcut_key,
        modifiers,
        current_ts_ms(),
    );
    state.pressed_non_modifier_keys.remove(physical_key_id);
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
    physical_key_id: String,
    shortcut_key: String,
    modifiers: ModifierSnapshot,
    is_key_combo: bool,
) {
    if let Ok(mut locked) = state.lock() {
        apply_collector_event(
            &mut locked,
            CollectorEvent::NonModifierKeyDown {
                physical_key_id,
                shortcut_key,
                modifiers,
                is_key_combo,
                capture_context: capture_context(),
                at: Instant::now(),
            },
        );
    }
}

fn on_non_modifier_key_up(
    state: &Arc<Mutex<CollectorState>>,
    physical_key_id: &str,
    shortcut_key: &str,
    modifiers: ModifierSnapshot,
) {
    if let Ok(mut locked) = state.lock() {
        apply_collector_event(
            &mut locked,
            CollectorEvent::NonModifierKeyUp {
                physical_key_id: physical_key_id.to_string(),
                shortcut_key: shortcut_key.to_string(),
                modifiers,
                capture_context: capture_context(),
            },
        );
    }
}

// Apply one collector event to state. This keeps runtime and test event semantics aligned.
fn apply_collector_event(state: &mut CollectorState, event: CollectorEvent) {
    match event {
        CollectorEvent::NonModifierKeyDown {
            physical_key_id,
            shortcut_key,
            modifiers,
            is_key_combo,
            capture_context,
            at,
        } => apply_non_modifier_key_down(
            state,
            physical_key_id,
            shortcut_key,
            modifiers,
            is_key_combo,
            capture_context,
            at,
        ),
        CollectorEvent::NonModifierKeyUp {
            physical_key_id,
            shortcut_key,
            modifiers,
            capture_context,
        } => apply_non_modifier_key_up(
            state,
            &physical_key_id,
            &shortcut_key,
            modifiers,
            &capture_context,
        ),
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
fn normalize_non_macos_key(key: rdev::Key) -> Option<String> {
    use rdev::Key;
    let normalized = match key {
        Key::KeyA => "a",
        Key::KeyB => "b",
        Key::KeyC => "c",
        Key::KeyD => "d",
        Key::KeyE => "e",
        Key::KeyF => "f",
        Key::KeyG => "g",
        Key::KeyH => "h",
        Key::KeyI => "i",
        Key::KeyJ => "j",
        Key::KeyK => "k",
        Key::KeyL => "l",
        Key::KeyM => "m",
        Key::KeyN => "n",
        Key::KeyO => "o",
        Key::KeyP => "p",
        Key::KeyQ => "q",
        Key::KeyR => "r",
        Key::KeyS => "s",
        Key::KeyT => "t",
        Key::KeyU => "u",
        Key::KeyV => "v",
        Key::KeyW => "w",
        Key::KeyX => "x",
        Key::KeyY => "y",
        Key::KeyZ => "z",
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        Key::Space => "space",
        Key::Return => "enter",
        Key::Tab => "tab",
        Key::Escape => "esc",
        Key::Backspace => "backspace",
        Key::Delete => "delete",
        Key::UpArrow => "up",
        Key::DownArrow => "down",
        Key::LeftArrow => "left",
        Key::RightArrow => "right",
        _ => return None,
    };
    Some(normalized.to_string())
}

#[cfg(not(target_os = "macos"))]
fn on_key_event_non_macos(state: &Arc<Mutex<CollectorState>>, key: rdev::Key, pressed: bool) {
    let (is_modifier_key, modifiers_before) = if let Ok(mut locked) = state.lock() {
        let is_modifier_key = ModifierState::is_modifier_key(key);
        let modifiers_before = locked.modifier_state.snapshot();
        locked.modifier_state.update(key, pressed);
        (is_modifier_key, modifiers_before)
    } else {
        return;
    };

    if is_modifier_key {
        return;
    }

    let shortcut_key =
        normalize_non_macos_key(key).unwrap_or_else(|| format!("{:?}", key).to_lowercase());
    let physical_key_id = format!("rdev:{:?}", key);
    if pressed {
        on_non_modifier_key_down(
            state,
            physical_key_id,
            shortcut_key,
            modifiers_before,
            modifiers_before.has_any(),
        );
    } else {
        on_non_modifier_key_up(state, &physical_key_id, &shortcut_key, modifiers_before);
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

    fn snapshot_from_macos_flags(flags: CGEventFlags) -> ModifierSnapshot {
        ModifierSnapshot {
            ctrl: flags & CG_EVENT_FLAG_MASK_CONTROL != 0,
            opt: flags & CG_EVENT_FLAG_MASK_ALTERNATE != 0,
            shift: flags & CG_EVENT_FLAG_MASK_SHIFT != 0,
            cmd: flags & CG_EVENT_FLAG_MASK_COMMAND != 0,
            function: flags & CG_EVENT_FLAG_MASK_SECONDARY_FN != 0,
        }
    }

    fn normalize_macos_keycode(key_code: i64) -> String {
        let key = match key_code {
            0 => "a",
            1 => "s",
            2 => "d",
            3 => "f",
            4 => "h",
            5 => "g",
            6 => "z",
            7 => "x",
            8 => "c",
            9 => "v",
            11 => "b",
            12 => "q",
            13 => "w",
            14 => "e",
            15 => "r",
            16 => "y",
            17 => "t",
            31 => "o",
            32 => "u",
            34 => "i",
            35 => "p",
            37 => "l",
            38 => "j",
            40 => "k",
            45 => "n",
            46 => "m",
            18 => "1",
            19 => "2",
            20 => "3",
            21 => "4",
            23 => "5",
            22 => "6",
            26 => "7",
            28 => "8",
            25 => "9",
            29 => "0",
            24 => "=",
            27 => "-",
            33 => "[",
            30 => "]",
            41 => ";",
            39 => "'",
            42 => "\\",
            43 => ",",
            47 => ".",
            44 => "/",
            36 => "enter",
            48 => "tab",
            49 => "space",
            51 => "backspace",
            53 => "esc",
            123 => "left",
            124 => "right",
            125 => "down",
            126 => "up",
            _ => return format!("k{key_code}"),
        };
        key.to_string()
    }

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
            let physical_key_id = format!("mac:{key_code}");
            let shortcut_key = normalize_macos_keycode(key_code);
            let modifiers = snapshot_from_macos_flags(flags);
            if type_ == CG_EVENT_TYPE_KEY_DOWN {
                on_non_modifier_key_down(
                    state,
                    physical_key_id,
                    shortcut_key,
                    modifiers,
                    modifiers.has_any(),
                );
            } else {
                on_non_modifier_key_up(state, &physical_key_id, &shortcut_key, modifiers);
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
    let shortcut_stats = snapshot_shortcut_rows(state);
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
        shortcut_stats,
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

pub fn set_shortcut_rules(
    state: &mut CollectorState,
    require_cmd_or_ctrl: bool,
    allow_alt_only: bool,
    min_modifiers: u8,
    allowlist: &[String],
    blocklist: &[String],
) {
    state.shortcut_require_cmd_or_ctrl = require_cmd_or_ctrl;
    state.shortcut_allow_alt_only = allow_alt_only;
    state.shortcut_min_modifiers = min_modifiers.max(1);
    state.shortcut_allowlist = allowlist
        .iter()
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
        .collect();
    state.shortcut_blocklist = blocklist
        .iter()
        .map(|v| v.trim().to_ascii_lowercase())
        .filter(|v| !v.is_empty())
        .collect();
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
    state.shortcut_usage.clear();
    state.event_chunks.clear();
    state.open_event_chunk = None;
    let _ = state.storage.save_stats(&state.stats);
    let analytics = build_stored_input_analytics(state);
    let _ = state.storage.save_input_analytics(&analytics);
}

// Build shortcut rows sorted by frequency for frontend leaderboard rendering.
fn snapshot_shortcut_rows(state: &CollectorState) -> Vec<ShortcutStatRow> {
    let mut rows: Vec<ShortcutStatRow> = state
        .shortcut_usage
        .iter()
        .map(|(shortcut_id, usage)| {
            let mut apps: Vec<ShortcutAppUsageRow> = usage
                .by_app
                .iter()
                .map(|(app_name, count)| ShortcutAppUsageRow {
                    app_name: app_name.clone(),
                    count: *count,
                })
                .collect();
            apps.sort_by(|a, b| {
                b.count
                    .cmp(&a.count)
                    .then_with(|| a.app_name.cmp(&b.app_name))
            });
            ShortcutStatRow {
                shortcut_id: shortcut_id.clone(),
                count: usage.count,
                apps: apps.into_iter().take(8).collect(),
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.shortcut_id.cmp(&b.shortcut_id))
    });
    rows
}

// Rebuild shortcut aggregate map from persisted key-down events.
fn rebuild_shortcut_usage_from_chunks(state: &mut CollectorState) {
    let mut aggregated: HashMap<String, ShortcutUsageValue> = HashMap::new();
    for chunk in &state.event_chunks {
        let app_id = state
            .app_dict
            .get(&chunk.app_ref)
            .cloned()
            .unwrap_or_else(|| format!("app:{}", chunk.app_ref));
        for raw_event in &chunk.events {
            let Some((_dt, event_type, key, modifiers)) = parse_compact_event(raw_event) else {
                continue;
            };
            if event_type != 'd' {
                continue;
            }
            let shortcut_id = normalize_shortcut_id(modifiers, &key);
            if !should_count_shortcut(state, modifiers, &shortcut_id) {
                continue;
            }
            let usage = aggregated
                .entry(shortcut_id)
                .or_insert_with(ShortcutUsageValue::default);
            usage.count = usage.count.saturating_add(1);
            *usage.by_app.entry(app_id.clone()).or_insert(0) += 1;
        }
    }
    state.shortcut_usage = aggregated;
}

// Parse compact event string `dt,t,k,m`; return None when format is invalid.
fn parse_compact_event(raw: &str) -> Option<(i64, char, String, ModifierSnapshot)> {
    let mut segments = raw.splitn(4, ',');
    let dt = segments.next()?.parse::<i64>().ok()?;
    let event_type = segments.next()?.chars().next()?;
    let key = segments.next()?.to_string();
    let modifier_mask = segments.next()?.parse::<u8>().ok()?;
    Some((dt, event_type, key, ModifierSnapshot::from_bitmask(modifier_mask)))
}

// Compute local [start,end) timestamp range in milliseconds by filter id.
fn shortcut_range_window_ms(range: &str, now_ms: i64) -> (i64, i64) {
    let now_local = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(now_ms)
        .map(|v| v.with_timezone(&Local))
        .unwrap_or_else(Local::now);
    let today = now_local.date_naive();
    let midnight_naive = today
        .and_hms_opt(0, 0, 0)
        .unwrap_or_else(|| now_local.naive_local());
    let today_start = Local
        .from_local_datetime(&midnight_naive)
        .single()
        .or_else(|| Local.from_local_datetime(&midnight_naive).earliest())
        .or_else(|| Local.from_local_datetime(&midnight_naive).latest())
        .unwrap_or(now_local)
        .timestamp_millis();
    let tomorrow_start = today_start + ChronoDuration::days(1).num_milliseconds();
    if range == "today" {
        return (today_start, tomorrow_start);
    }
    if range == "yesterday" {
        let yesterday_start = today_start - ChronoDuration::days(1).num_milliseconds();
        return (yesterday_start, today_start);
    }
    let seven_days_start = today_start - ChronoDuration::days(6).num_milliseconds();
    (seven_days_start, tomorrow_start)
}

// Rebuild shortcut usage rows from compact events for a requested time window.
fn snapshot_shortcut_rows_in_window(
    state: &CollectorState,
    start_ms: i64,
    end_ms: i64,
) -> Vec<ShortcutStatRow> {
    let mut aggregated: HashMap<String, ShortcutUsageValue> = HashMap::new();
    let mut consume_chunk = |chunk_start_ms: i64, app_ref: u32, events: &[String]| {
        let app_id = state
            .app_dict
            .get(&app_ref)
            .cloned()
            .unwrap_or_else(|| format!("app:{app_ref}"));
        for raw_event in events {
            let Some((dt, event_type, key, modifiers)) = parse_compact_event(raw_event) else {
                continue;
            };
            if event_type != 'd' {
                continue;
            }
            let event_ms = chunk_start_ms.saturating_add(dt.max(0));
            if event_ms < start_ms || event_ms >= end_ms {
                continue;
            }
            let shortcut_id = normalize_shortcut_id(modifiers, &key);
            if !should_count_shortcut(state, modifiers, &shortcut_id) {
                continue;
            }
            let usage = aggregated
                .entry(shortcut_id)
                .or_insert_with(ShortcutUsageValue::default);
            usage.count = usage.count.saturating_add(1);
            *usage.by_app.entry(app_id.clone()).or_insert(0) += 1;
        }
    };
    for chunk in &state.event_chunks {
        consume_chunk(chunk.chunk_start_ms, chunk.app_ref, &chunk.events);
    }
    if let Some(open_chunk) = state.open_event_chunk.as_ref() {
        consume_chunk(open_chunk.chunk_start_ms, open_chunk.app_ref, &open_chunk.events);
    }
    let mut rows: Vec<ShortcutStatRow> = aggregated
        .into_iter()
        .map(|(shortcut_id, usage)| {
            let mut apps: Vec<ShortcutAppUsageRow> = usage
                .by_app
                .into_iter()
                .map(|(app_name, count)| ShortcutAppUsageRow { app_name, count })
                .collect();
            apps.sort_by(|a, b| {
                b.count
                    .cmp(&a.count)
                    .then_with(|| a.app_name.cmp(&b.app_name))
            });
            ShortcutStatRow {
                shortcut_id,
                count: usage.count,
                apps: apps.into_iter().take(8).collect(),
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.shortcut_id.cmp(&b.shortcut_id))
    });
    rows
}

/// Build shortcut leaderboard rows by selected range: `today` / `yesterday` / `7d`.
pub fn snapshot_shortcut_rows_by_range(state: &CollectorState, range: &str) -> Vec<ShortcutStatRow> {
    let now_ms = current_ts_ms();
    let (start_ms, end_ms) = shortcut_range_window_ms(range, now_ms);
    snapshot_shortcut_rows_in_window(state, start_ms, end_ms)
}

fn current_minute() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        apply_collector_event, set_ignore_key_combos, should_ignore_keypress, snapshot,
        snapshot_rows, CaptureContext, CollectorEvent, CollectorState, ModifierSnapshot, StatsKey,
        StatsValue,
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
            shortcut_usage: HashMap::new(),
            app_dict: HashMap::new(),
            app_ref_by_app: HashMap::new(),
            next_app_ref: 1,
            event_chunks: Vec::new(),
            open_event_chunk: None,
            shortcut_require_cmd_or_ctrl: true,
            shortcut_allow_alt_only: false,
            shortcut_min_modifiers: 1,
            shortcut_allowlist: HashSet::new(),
            shortcut_blocklist: HashSet::new(),
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
                physical_key_id: key_id.to_string(),
                shortcut_key: key_id.to_string(),
                modifiers: ModifierSnapshot::default(),
                is_key_combo,
                capture_context: self.default_context.clone(),
                at,
            });
        }

        // Push key-up for one key id.
        fn key_up(&mut self, key_id: &str) {
            self.push(CollectorEvent::NonModifierKeyUp {
                physical_key_id: key_id.to_string(),
                shortcut_key: key_id.to_string(),
                modifiers: ModifierSnapshot::default(),
                capture_context: self.default_context.clone(),
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

        // Push one tick event with a custom context, useful for auto-pause tests.
        fn tick_with_context(
            &mut self,
            elapsed: Duration,
            at: Instant,
            capture_context: CaptureContext,
        ) {
            self.push(CollectorEvent::Tick {
                elapsed,
                capture_context,
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

    #[test]
    fn auto_pause_tick_resets_pressed_state_until_new_key_down() {
        let mut harness = CollectorEventHarness::new();
        let now = Instant::now();

        harness.key_down("k:a", false, now);
        harness.tick(Duration::from_millis(500), now + Duration::from_millis(500));
        harness.tick_with_context(
            Duration::from_millis(300),
            now + Duration::from_millis(800),
            CaptureContext {
                app_name: "Editor".to_string(),
                window_title: "Doc".to_string(),
                bundle_id: Some("com.test.editor".to_string()),
                secure_input: true,
            },
        );
        harness.tick(
            Duration::from_millis(400),
            now + Duration::from_millis(1200),
        );
        harness.key_down("k:a", false, now + Duration::from_millis(1300));
        harness.tick(
            Duration::from_millis(200),
            now + Duration::from_millis(1500),
        );

        let rows = harness.rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].key_count, 2);
        assert_eq!(rows[0].active_typing_ms, 700);
    }

    #[test]
    fn ignore_key_combo_event_is_not_counted_in_event_stream() {
        let mut harness = CollectorEventHarness::new();
        let now = Instant::now();
        set_ignore_key_combos(&mut harness.state, true);

        harness.key_down("k:a", true, now);
        harness.tick(Duration::from_millis(300), now + Duration::from_millis(300));
        harness.key_down("k:a", false, now + Duration::from_millis(400));
        harness.tick(Duration::from_millis(200), now + Duration::from_millis(600));

        let rows = harness.rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].key_count, 1);
        assert_eq!(rows[0].active_typing_ms, 200);
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
        physical_key_id: String,
        shortcut_key: String,
        modifiers: ModifierSnapshot,
        is_key_combo: bool,
        capture_context: CaptureContext,
        at: Instant,
    },
    NonModifierKeyUp {
        physical_key_id: String,
        shortcut_key: String,
        modifiers: ModifierSnapshot,
        capture_context: CaptureContext,
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
