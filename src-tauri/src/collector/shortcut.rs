//! Shortcut analytics module.
//! Owns shortcut normalization/filtering and compact input-event chunk persistence.

use std::collections::HashMap;

use chrono::{Duration as ChronoDuration, Local, TimeZone};

use crate::storage::{StoredInputAnalytics, StoredInputEventChunk, StoredShortcutUsage};

use super::{
    current_ts_ms, CaptureContext, CollectorState, InputEventChunk, ModifierSnapshot,
    OpenInputEventChunk, ShortcutAppUsageRow, ShortcutStatRow, ShortcutUsageValue,
};

const INPUT_CHUNK_WINDOW_MS: i64 = 5_000;
const INPUT_CHUNK_MAX_EVENTS: usize = 500;
const INPUT_CHUNK_MAX_STORED: usize = 20_000;

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
pub(super) fn flush_expired_open_chunk(state: &mut CollectorState, now_ms: i64) {
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
pub(super) fn append_input_event(
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

pub(super) fn update_shortcut_usage(
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

pub(super) fn build_stored_input_analytics(state: &mut CollectorState) -> StoredInputAnalytics {
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

// Build shortcut rows sorted by frequency for frontend leaderboard rendering.
pub(super) fn snapshot_shortcut_rows(state: &CollectorState) -> Vec<ShortcutStatRow> {
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

pub(super) fn rebuild_shortcut_usage_from_chunks(state: &mut CollectorState) {
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
    Some((
        dt,
        event_type,
        key,
        ModifierSnapshot::from_bitmask(modifier_mask),
    ))
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
        consume_chunk(
            open_chunk.chunk_start_ms,
            open_chunk.app_ref,
            &open_chunk.events,
        );
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
pub fn snapshot_shortcut_rows_by_range(
    state: &CollectorState,
    range: &str,
) -> Vec<ShortcutStatRow> {
    let now_ms = current_ts_ms();
    let (start_ms, end_ms) = shortcut_range_window_ms(range, now_ms);
    snapshot_shortcut_rows_in_window(state, start_ms, end_ms)
}
