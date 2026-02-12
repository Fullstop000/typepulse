//! Collector state API module.
//! Exposes snapshot/build and runtime mutation entrypoints used by commands/UI.

use crate::app_config::MenuBarDisplayMode;

use super::{
    build_stored_input_analytics, reset_active_typing_state, snapshot_shortcut_rows,
    CollectorState, StatsRow, StatsSnapshot,
};

/// Build sorted row snapshots from in-memory collector stats.
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

/// Build the frontend snapshot payload from current runtime collector state.
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

/// Pause/resume collector runtime and clear active key states when pausing.
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

/// Update shortcut counting rules used by runtime aggregation.
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

/// Clear all collected stats and persist cleared payload back to storage.
pub fn clear_stats(state: &mut CollectorState) {
    state.stats.clear();
    state.shortcut_usage.clear();
    state.event_chunks.clear();
    state.open_event_chunk = None;
    let _ = state.storage.save_stats(&state.stats);
    let analytics = build_stored_input_analytics(state);
    let _ = state.storage.save_input_analytics(&analytics);
}
