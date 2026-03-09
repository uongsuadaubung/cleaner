use chrono::{DateTime, Local};
use std::time::SystemTime;

/// Format bytes thành dạng human-readable (KB, MB, GB)
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format SystemTime thành chuỗi ngày tháng
pub fn format_date(time: Option<SystemTime>) -> String {
    match time {
        Some(t) => {
            let datetime: DateTime<Local> = t.into();
            datetime.format("%Y-%m-%d %H:%M").to_string()
        }
        None => "N/A".to_string(),
    }
}

/// Kiểm tra file có cũ hơn N ngày không
pub fn is_older_than_days(time: Option<SystemTime>, days: u64) -> bool {
    match time {
        Some(t) => {
            let duration = SystemTime::now().duration_since(t).unwrap_or_default();
            duration.as_secs() > days * 24 * 60 * 60
        }
        None => false,
    }
}
