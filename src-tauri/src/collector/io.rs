//! Collector file I/O module.
//! Handles CSV/debug log persistence only; business aggregation stays elsewhere.

use std::{fs::File, io::Write, path::PathBuf};

use chrono::Local;

use super::StatsRow;

// Persist aggregated rows into CSV for external inspection/debugging.
pub(super) fn write_csv(path: &PathBuf, rows: &[StatsRow]) -> Result<(), String> {
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

/// Append one line into runtime app log file.
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
