use std::{collections::HashMap, path::PathBuf};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::collector::{StatsKey, StatsValue};

#[derive(Serialize, Deserialize, Clone)]
struct StoredRow {
    date: String,
    app_name: String,
    window_title: String,
    active_typing_ms: u64,
    key_count: u64,
    session_count: u64,
}

/// Persisted shortcut aggregation for one normalized shortcut id.
#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct StoredShortcutUsage {
    pub(crate) count: u64,
    #[serde(default)]
    pub(crate) by_app: HashMap<String, u64>,
}

/// Persisted input-event chunk with compact string events: `dt,t,k,m`.
#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct StoredInputEventChunk {
    pub(crate) v: u8,
    pub(crate) chunk_start_ms: i64,
    pub(crate) app_ref: u32,
    #[serde(default)]
    pub(crate) events: Vec<String>,
}

/// Persisted analytics payload for shortcut usage and optional event replay chunks.
#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct StoredInputAnalytics {
    #[serde(default)]
    pub(crate) shortcut_usage: HashMap<String, StoredShortcutUsage>,
    #[serde(default)]
    pub(crate) app_dict: HashMap<u32, String>,
    #[serde(default)]
    pub(crate) next_app_ref: u32,
    #[serde(default)]
    pub(crate) event_chunks: Vec<StoredInputEventChunk>,
}

pub(crate) trait DetailStorage: Send + Sync {
    fn load_stats(&self) -> Result<HashMap<StatsKey, StatsValue>, String>;
    fn save_stats(&self, stats: &HashMap<StatsKey, StatsValue>) -> Result<(), String>;
    fn load_input_analytics(&self) -> Result<StoredInputAnalytics, String>;
    fn save_input_analytics(&self, analytics: &StoredInputAnalytics) -> Result<(), String>;
}

pub(crate) struct JsonFileStorage {
    pub(crate) path: PathBuf,
}

impl JsonFileStorage {
    fn base_name(&self) -> Option<String> {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
    }

    fn date_prefix(date: &str) -> Option<String> {
        if date.len() < 10 {
            return None;
        }
        Some(date[..10].to_string())
    }

    fn dated_path(&self, date_prefix: &str) -> Option<PathBuf> {
        let parent = self.path.parent()?;
        let base = self.base_name()?;
        Some(parent.join(format!("{date_prefix}-{base}")))
    }

    fn stats_to_rows(stats: &HashMap<StatsKey, StatsValue>) -> Vec<StoredRow> {
        let mut rows: Vec<StoredRow> = stats
            .iter()
            .map(|(key, value)| StoredRow {
                date: key.date.clone(),
                app_name: key.app_name.clone(),
                window_title: key.window_title.clone(),
                active_typing_ms: value.active_typing_ms,
                key_count: value.key_count,
                session_count: value.session_count,
            })
            .collect();
        rows.sort_by(|a, b| {
            (&a.date, &a.app_name, &a.window_title).cmp(&(&b.date, &b.app_name, &b.window_title))
        });
        rows
    }

    fn rows_to_stats(rows: Vec<StoredRow>) -> HashMap<StatsKey, StatsValue> {
        let mut stats: HashMap<StatsKey, StatsValue> = HashMap::new();
        for row in rows {
            let key = StatsKey {
                date: row.date,
                app_name: row.app_name,
                window_title: row.window_title,
            };
            let entry = stats.entry(key).or_insert(StatsValue {
                active_typing_ms: 0,
                key_count: 0,
                session_count: 0,
            });
            entry.active_typing_ms += row.active_typing_ms;
            entry.key_count += row.key_count;
            entry.session_count += row.session_count;
        }
        stats
    }

    fn analytics_path(&self) -> Option<PathBuf> {
        let parent = self.path.parent()?;
        let base = self.base_name()?;
        Some(parent.join(format!("analytics-{base}")))
    }

    fn analytics_daily_suffix(&self) -> Option<String> {
        let base = self.base_name()?;
        Some(format!("-analytics-{base}"))
    }

    fn analytics_dated_path(&self, date_prefix: &str) -> Option<PathBuf> {
        let parent = self.path.parent()?;
        let base = self.base_name()?;
        Some(parent.join(format!("{date_prefix}-analytics-{base}")))
    }

    // Convert event timestamp to local date prefix for per-day analytics file sharding.
    fn date_prefix_from_timestamp_ms(timestamp_ms: i64) -> Option<String> {
        let dt = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(timestamp_ms)?;
        Some(
            dt.with_timezone(&Local)
                .format("%Y-%m-%d")
                .to_string(),
        )
    }

    // Parse both legacy `Vec<StoredRow>` and structured payload with `rows` field.
    fn parse_rows_content(content: &str) -> Result<Vec<StoredRow>, String> {
        #[derive(Deserialize)]
        struct StructuredRows {
            #[serde(default)]
            rows: Vec<StoredRow>,
        }

        if let Ok(rows) = serde_json::from_str::<Vec<StoredRow>>(content) {
            return Ok(rows);
        }
        let structured: StructuredRows =
            serde_json::from_str(content).map_err(|e| e.to_string())?;
        Ok(structured.rows)
    }

    // Merge analytics payload into accumulator, summing usage and app-level counters.
    fn merge_analytics(into: &mut StoredInputAnalytics, from: StoredInputAnalytics) {
        for (shortcut_id, usage) in from.shortcut_usage {
            let entry = into
                .shortcut_usage
                .entry(shortcut_id)
                .or_insert_with(StoredShortcutUsage::default);
            entry.count = entry.count.saturating_add(usage.count);
            for (app_id, count) in usage.by_app {
                let app_entry = entry.by_app.entry(app_id).or_insert(0);
                *app_entry = app_entry.saturating_add(count);
            }
        }
        for (app_ref, app_id) in from.app_dict {
            into.app_dict.entry(app_ref).or_insert(app_id);
        }
        into.next_app_ref = into.next_app_ref.max(from.next_app_ref);
        into.event_chunks.extend(from.event_chunks);
    }
}

impl DetailStorage for JsonFileStorage {
    fn load_stats(&self) -> Result<HashMap<StatsKey, StatsValue>, String> {
        let mut rows: Vec<StoredRow> = Vec::new();
        // Read legacy monolithic storage file first, if it exists.
        let legacy_content = std::fs::read_to_string(&self.path);
        match legacy_content {
            Ok(content) => {
                let mut legacy_rows = Self::parse_rows_content(&content)?;
                rows.append(&mut legacy_rows);
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err.to_string()),
        }
        let parent = match self.path.parent() {
            Some(parent) => parent,
            None => return Ok(Self::rows_to_stats(rows)),
        };
        let base = match self.base_name() {
            Some(base) => base,
            None => return Ok(Self::rows_to_stats(rows)),
        };
        // Merge all daily rotated files that match the base filename.
        let entries = match std::fs::read_dir(parent) {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::rows_to_stats(rows))
            }
            Err(err) => return Err(err.to_string()),
        };
        let suffix = format!("-{base}");
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = match path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => continue,
            };
            if !file_name.ends_with(&suffix) {
                continue;
            }
            // Skip files that cannot be parsed; keep loading what we can.
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mut day_rows) = Self::parse_rows_content(&content) {
                    rows.append(&mut day_rows);
                }
            }
        }
        Ok(Self::rows_to_stats(rows))
    }

    fn save_stats(&self, stats: &HashMap<StatsKey, StatsValue>) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let rows = Self::stats_to_rows(stats);
        let mut grouped: HashMap<String, Vec<StoredRow>> = HashMap::new();
        for row in rows {
            if let Some(date_prefix) = Self::date_prefix(&row.date) {
                grouped.entry(date_prefix).or_default().push(row);
            }
        }
        // Write each date bucket into its own file for easier rotation.
        for (date_prefix, day_rows) in grouped {
            let path = match self.dated_path(&date_prefix) {
                Some(path) => path,
                None => continue,
            };
            let bytes = serde_json::to_vec(&day_rows).map_err(|e| e.to_string())?;
            let tmp_path = path.with_extension("json.tmp");
            std::fs::write(&tmp_path, bytes).map_err(|e| e.to_string())?;
            std::fs::rename(&tmp_path, &path).map_err(|e| e.to_string())?;
        }
        // Remove legacy monolithic file once daily files are written.
        let _ = std::fs::remove_file(&self.path);
        Ok(())
    }

    fn load_input_analytics(&self) -> Result<StoredInputAnalytics, String> {
        let mut merged = StoredInputAnalytics::default();
        // Load legacy monolithic analytics file for backward compatibility.
        if let Some(path) = self.analytics_path() {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let payload: StoredInputAnalytics =
                        serde_json::from_str(&content).map_err(|e| e.to_string())?;
                    Self::merge_analytics(&mut merged, payload);
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => return Err(err.to_string()),
            }
        }
        // Merge all daily analytics files.
        let parent = match self.path.parent() {
            Some(parent) => parent,
            None => return Ok(merged),
        };
        let suffix = match self.analytics_daily_suffix() {
            Some(suffix) => suffix,
            None => return Ok(merged),
        };
        let entries = match std::fs::read_dir(parent) {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(merged),
            Err(err) => return Err(err.to_string()),
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = match path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => continue,
            };
            if !file_name.ends_with(&suffix) {
                continue;
            }
            // Keep loading other files even if one daily file is corrupted.
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(payload) = serde_json::from_str::<StoredInputAnalytics>(&content) {
                    Self::merge_analytics(&mut merged, payload);
                }
            }
        }
        merged
            .event_chunks
            .sort_by(|a, b| a.chunk_start_ms.cmp(&b.chunk_start_ms));
        Ok(merged)
    }

    fn save_input_analytics(&self, analytics: &StoredInputAnalytics) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let parent = match self.path.parent() {
            Some(parent) => parent,
            None => return Ok(()),
        };
        let suffix = match self.analytics_daily_suffix() {
            Some(suffix) => suffix,
            None => return Ok(()),
        };
        let mut grouped_chunks: HashMap<String, Vec<StoredInputEventChunk>> = HashMap::new();
        for chunk in &analytics.event_chunks {
            let date_prefix = match Self::date_prefix_from_timestamp_ms(chunk.chunk_start_ms) {
                Some(value) => value,
                None => continue,
            };
            grouped_chunks
                .entry(date_prefix)
                .or_default()
                .push(chunk.clone());
        }
        // Remove stale daily analytics files before writing the new set.
        let entries = match std::fs::read_dir(parent) {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(err.to_string()),
        };
        let next_dates: std::collections::HashSet<String> =
            grouped_chunks.keys().cloned().collect();
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = match path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            if !file_name.ends_with(&suffix) {
                continue;
            }
            let date_prefix = file_name
                .strip_suffix(&suffix)
                .unwrap_or_default()
                .to_string();
            if !next_dates.contains(&date_prefix) {
                std::fs::remove_file(&path).map_err(|e| e.to_string())?;
            }
        }
        // Write one analytics payload per day. Shortcut usage can be rebuilt from chunks.
        for (date_prefix, chunks) in grouped_chunks {
            let path = match self.analytics_dated_path(&date_prefix) {
                Some(path) => path,
                None => continue,
            };
            let mut app_refs: std::collections::HashSet<u32> = std::collections::HashSet::new();
            for chunk in &chunks {
                app_refs.insert(chunk.app_ref);
            }
            let app_dict = analytics
                .app_dict
                .iter()
                .filter_map(|(app_ref, app_id)| {
                    if app_refs.contains(app_ref) {
                        Some((*app_ref, app_id.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            let payload = StoredInputAnalytics {
                shortcut_usage: HashMap::new(),
                app_dict,
                next_app_ref: analytics.next_app_ref,
                event_chunks: chunks,
            };
            let bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
            let tmp_path = path.with_extension("json.tmp");
            std::fs::write(&tmp_path, bytes).map_err(|e| e.to_string())?;
            std::fs::rename(&tmp_path, &path).map_err(|e| e.to_string())?;
        }
        // Remove legacy monolithic analytics file after daily files are written.
        if let Some(path) = self.analytics_path() {
            let _ = std::fs::remove_file(path);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{DetailStorage, JsonFileStorage, StoredInputAnalytics, StoredInputEventChunk};
    use crate::collector::{StatsKey, StatsValue};
    use std::{collections::HashMap, fs, path::PathBuf, time::SystemTime};

    fn temp_path(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let stamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("typepulse-{name}-{stamp}.json"));
        path
    }

    fn dated_path(base: &PathBuf, date: &str) -> Option<PathBuf> {
        let parent = base.parent()?;
        let name = base.file_name()?.to_str()?;
        Some(parent.join(format!("{date}-{name}")))
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let path = temp_path("missing");
        let storage = JsonFileStorage { path };
        let loaded = storage.load_stats().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn save_and_load_round_trip() {
        let path = temp_path("roundtrip");
        let storage = JsonFileStorage { path: path.clone() };
        let mut stats = HashMap::new();
        stats.insert(
            StatsKey {
                date: "2026-02-09 10:00".to_string(),
                app_name: "AppA".to_string(),
                window_title: "WindowA".to_string(),
            },
            StatsValue {
                active_typing_ms: 1200,
                key_count: 12,
                session_count: 2,
            },
        );
        stats.insert(
            StatsKey {
                date: "2026-02-10 10:01".to_string(),
                app_name: "AppB".to_string(),
                window_title: "WindowB".to_string(),
            },
            StatsValue {
                active_typing_ms: 800,
                key_count: 8,
                session_count: 1,
            },
        );
        storage.save_stats(&stats).unwrap();
        let loaded = storage.load_stats().unwrap();
        assert_eq!(loaded.len(), 2);
        let value = loaded
            .get(&StatsKey {
                date: "2026-02-09 10:00".to_string(),
                app_name: "AppA".to_string(),
                window_title: "WindowA".to_string(),
            })
            .unwrap();
        assert_eq!(value.active_typing_ms, 1200);
        assert_eq!(value.key_count, 12);
        assert_eq!(value.session_count, 2);
        if let Some(path) = dated_path(&path, "2026-02-09") {
            let _ = fs::remove_file(path);
        }
        if let Some(path) = dated_path(&path, "2026-02-10") {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn analytics_round_trip() {
        let path = temp_path("analytics");
        let storage = JsonFileStorage { path: path.clone() };
        let mut analytics = StoredInputAnalytics::default();
        analytics.app_dict.insert(1, "com.test.editor".to_string());
        analytics.next_app_ref = 2;
        analytics.event_chunks.push(StoredInputEventChunk {
            v: 1,
            chunk_start_ms: 1_706_054_400_000,
            app_ref: 1,
            events: vec!["0,d,c,8".to_string()],
        });
        storage.save_input_analytics(&analytics).unwrap();
        let loaded = storage.load_input_analytics().unwrap();
        assert_eq!(loaded.next_app_ref, 2);
        assert_eq!(loaded.event_chunks.len(), 1);
        assert_eq!(loaded.event_chunks[0].events.len(), 1);
        assert!(loaded.shortcut_usage.is_empty());
        if let Some(parent) = path.parent() {
            let _ = fs::remove_file(parent.join("2024-01-01-analytics-typepulse-analytics.json"));
        }
    }
}
