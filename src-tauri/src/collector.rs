//! Collector core module.
//! Owns runtime state, lifecycle bootstrap, and cross-module wiring.

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use serde::Serialize;

use crate::app_config::{AppConfig, MenuBarDisplayMode};
use crate::storage::{DetailStorage, JsonFileStorage, StoredInputAnalytics};

mod context;
mod events;
mod io;
mod listener;
mod shortcut;
mod state_api;

use self::context::{capture_context, CaptureContext, CollectorEvent};
use self::events::{
    apply_collector_event, on_non_modifier_key_down, on_non_modifier_key_up,
    reset_active_typing_state,
};

pub use self::context::{bundle_id_from_app_path, running_apps, RunningAppInfo};
#[cfg(test)]
use self::events::should_ignore_keypress;
pub use self::io::append_app_log;
use self::io::write_csv;
#[cfg(target_os = "macos")]
use self::listener::listen_keypress_macos;
#[cfg(not(target_os = "macos"))]
use self::listener::on_key_event_non_macos;
pub use self::shortcut::snapshot_shortcut_rows_by_range;
use self::shortcut::{
    build_stored_input_analytics, flush_expired_open_chunk, rebuild_shortcut_usage_from_chunks,
    snapshot_shortcut_rows,
};
pub use self::state_api::{
    add_excluded_bundle_id, clear_stats, remove_excluded_bundle_id, set_excluded_bundle_ids,
    set_ignore_key_combos, set_menu_bar_display_mode, set_one_password_suggestion_pending,
    set_paused, set_shortcut_rules, snapshot, snapshot_rows,
};

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

fn current_ts_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
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
