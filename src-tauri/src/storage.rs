use std::{collections::HashMap, path::PathBuf};

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

#[derive(Serialize, Deserialize, Clone, Default)]
pub(crate) struct AppConfig {
    #[serde(default)]
    pub(crate) ignore_key_combos: bool,
}

pub(crate) trait DetailStorage: Send + Sync {
    fn load_stats(&self) -> Result<HashMap<StatsKey, StatsValue>, String>;
    fn save_stats(&self, stats: &HashMap<StatsKey, StatsValue>) -> Result<(), String>;
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
}

impl DetailStorage for JsonFileStorage {
    fn load_stats(&self) -> Result<HashMap<StatsKey, StatsValue>, String> {
        let mut rows: Vec<StoredRow> = Vec::new();
        // Read legacy monolithic storage file first, if it exists.
        let legacy_content = std::fs::read_to_string(&self.path);
        match legacy_content {
            Ok(content) => {
                let mut legacy_rows: Vec<StoredRow> =
                    serde_json::from_str(&content).map_err(|e| e.to_string())?;
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
                if let Ok(mut day_rows) = serde_json::from_str::<Vec<StoredRow>>(&content) {
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
}

pub(crate) fn load_app_config(path: &PathBuf) -> Result<AppConfig, String> {
    match std::fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).map_err(|e| e.to_string()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
        Err(err) => Err(err.to_string()),
    }
}

pub(crate) fn save_app_config(path: &PathBuf, config: &AppConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let bytes = serde_json::to_vec_pretty(config).map_err(|e| e.to_string())?;
    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_path, path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DetailStorage, JsonFileStorage};
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
}
