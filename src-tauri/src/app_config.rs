use std::{path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub(crate) struct AppConfig {
    /// 是否忽略组合键（ctrl/alt/shift/cmd/fn + 其他键）。
    pub(crate) ignore_key_combos: bool,
    /// 采集线程的轮询周期（秒），越小实时性越高，CPU 唤醒更频繁。
    pub(crate) collector_tick_interval_secs: u64,
    /// 明细与 CSV 的刷盘周期（秒），越小数据越及时，磁盘写入更频繁。
    pub(crate) flush_interval_secs: u64,
    /// 两次按键间隔不超过该值（秒）时，计入活跃打字时长；超过则视为新会话。
    pub(crate) session_gap_secs: u64,
    /// 托盘摘要信息刷新周期（秒），越小显示越及时。
    pub(crate) tray_update_interval_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ignore_key_combos: false,
            collector_tick_interval_secs: 1,
            flush_interval_secs: 60,
            session_gap_secs: 5,
            tray_update_interval_secs: 1,
        }
    }
}

impl AppConfig {
    pub(crate) fn collector_tick_interval(&self) -> Duration {
        Duration::from_secs(self.collector_tick_interval_secs.max(1))
    }

    pub(crate) fn flush_interval(&self) -> Duration {
        Duration::from_secs(self.flush_interval_secs.max(1))
    }

    pub(crate) fn session_gap(&self) -> Duration {
        Duration::from_secs(self.session_gap_secs.max(1))
    }

    pub(crate) fn tray_update_interval(&self) -> Duration {
        Duration::from_secs(self.tray_update_interval_secs.max(1))
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
