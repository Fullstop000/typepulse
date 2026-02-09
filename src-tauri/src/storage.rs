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

pub(crate) trait DetailStorage: Send + Sync {
    fn load_stats(&self) -> Result<HashMap<StatsKey, StatsValue>, String>;
    fn save_stats(&self, stats: &HashMap<StatsKey, StatsValue>) -> Result<(), String>;
}

pub(crate) struct JsonFileStorage {
    pub(crate) path: PathBuf,
}

impl JsonFileStorage {
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
        let content = match std::fs::read_to_string(&self.path) {
            Ok(v) => v,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(HashMap::new()),
            Err(err) => return Err(err.to_string()),
        };
        let rows: Vec<StoredRow> = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(Self::rows_to_stats(rows))
    }

    fn save_stats(&self, stats: &HashMap<StatsKey, StatsValue>) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let rows = Self::stats_to_rows(stats);
        let bytes = serde_json::to_vec(&rows).map_err(|e| e.to_string())?;
        let tmp_path = self.path.with_extension("json.tmp");
        std::fs::write(&tmp_path, bytes).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp_path, &self.path).map_err(|e| e.to_string())?;
        Ok(())
    }
}
