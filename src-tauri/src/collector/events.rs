//! Event state-machine module.
//! Applies key/tick events to runtime state and maintains typing/session semantics.

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use chrono::Local;

use super::context::{auto_pause_reason, is_auto_paused};
use super::shortcut::{append_input_event, update_shortcut_usage};
use super::{
    capture_context, current_ts_ms, CaptureContext, CollectorEvent, CollectorState,
    ModifierSnapshot, StatsKey, StatsValue,
};

// Reset runtime key states when capture is paused to avoid stale key-down state.
pub(super) fn reset_active_typing_state(state: &mut CollectorState) {
    state.pressed_non_modifier_keys.clear();
    state.active_stats_key = None;
    #[cfg(not(target_os = "macos"))]
    {
        state.modifier_state = super::ModifierState::default();
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

fn current_minute() -> String {
    Local::now().format("%Y-%m-%d %H:%M").to_string()
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

pub(super) fn on_non_modifier_key_down(
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

pub(super) fn on_non_modifier_key_up(
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
pub(super) fn apply_collector_event(state: &mut CollectorState, event: CollectorEvent) {
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

pub(super) fn should_ignore_keypress(ignore_key_combos: bool, is_key_combo: bool) -> bool {
    ignore_key_combos && is_key_combo
}
