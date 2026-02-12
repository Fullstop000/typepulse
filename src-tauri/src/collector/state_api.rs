//! Collector state API module.
//! Implements `CollectorState` methods for snapshot building and state mutations.

use crate::app_config::MenuBarDisplayMode;

use super::{
    build_stored_input_analytics, reset_active_typing_state, snapshot_shortcut_rows,
    CollectorState, StatsRow, StatsSnapshot,
};

impl CollectorState {
    /// Build sorted row snapshots from in-memory collector stats.
    pub fn snapshot_rows(&self) -> Result<Vec<StatsRow>, String> {
        let mut rows: Vec<StatsRow> = self
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
    pub fn snapshot(&self) -> StatsSnapshot {
        let rows = self.snapshot_rows().unwrap_or_default();
        let shortcut_stats = snapshot_shortcut_rows(self);
        let mut excluded_bundle_ids: Vec<String> =
            self.excluded_bundle_ids.iter().cloned().collect();
        excluded_bundle_ids.sort();
        StatsSnapshot {
            rows,
            paused: self.paused,
            auto_paused: self.auto_paused,
            auto_pause_reason: self.auto_pause_reason.clone(),
            keyboard_active: self.keyboard_active,
            ignore_key_combos: self.ignore_key_combos,
            excluded_bundle_ids,
            one_password_suggestion_pending: self.one_password_suggestion_pending,
            tray_display_mode: self.menu_bar_display_mode.as_str().to_string(),
            last_error: self.last_error.clone(),
            log_path: self.log_path.to_string_lossy().to_string(),
            shortcut_stats,
        }
    }

    /// Pause/resume collector runtime and clear active key states when pausing.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        if paused {
            reset_active_typing_state(self);
        }
    }

    pub fn set_ignore_key_combos(&mut self, ignore_key_combos: bool) {
        self.ignore_key_combos = ignore_key_combos;
    }

    pub fn set_menu_bar_display_mode(&mut self, mode: MenuBarDisplayMode) {
        self.menu_bar_display_mode = mode;
    }

    /// Update shortcut counting rules used by runtime aggregation.
    pub fn set_shortcut_rules(
        &mut self,
        require_cmd_or_ctrl: bool,
        allow_alt_only: bool,
        min_modifiers: u8,
        allowlist: &[String],
        blocklist: &[String],
    ) {
        self.shortcut_require_cmd_or_ctrl = require_cmd_or_ctrl;
        self.shortcut_allow_alt_only = allow_alt_only;
        self.shortcut_min_modifiers = min_modifiers.max(1);
        self.shortcut_allowlist = allowlist
            .iter()
            .map(|v| v.trim().to_ascii_lowercase())
            .filter(|v| !v.is_empty())
            .collect();
        self.shortcut_blocklist = blocklist
            .iter()
            .map(|v| v.trim().to_ascii_lowercase())
            .filter(|v| !v.is_empty())
            .collect();
    }

    pub fn set_excluded_bundle_ids(&mut self, bundle_ids: &[String]) {
        self.excluded_bundle_ids = bundle_ids
            .iter()
            .map(|v| v.trim().to_ascii_lowercase())
            .filter(|v| !v.is_empty())
            .collect();
    }

    pub fn add_excluded_bundle_id(&mut self, bundle_id: &str) -> bool {
        let normalized = bundle_id.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return false;
        }
        self.excluded_bundle_ids.insert(normalized)
    }

    pub fn remove_excluded_bundle_id(&mut self, bundle_id: &str) -> bool {
        self.excluded_bundle_ids
            .remove(&bundle_id.trim().to_ascii_lowercase())
    }

    pub fn set_one_password_suggestion_pending(&mut self, pending: bool) {
        self.one_password_suggestion_pending = pending;
    }

    /// Clear all collected stats and persist cleared payload back to storage.
    pub fn clear_stats(&mut self) {
        self.stats.clear();
        self.shortcut_usage.clear();
        self.event_chunks.clear();
        self.open_event_chunk = None;
        let _ = self.storage.save_stats(&self.stats);
        let analytics = build_stored_input_analytics(self);
        let _ = self.storage.save_input_analytics(&analytics);
    }
}
