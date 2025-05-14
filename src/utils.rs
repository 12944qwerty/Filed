use std::time::SystemTime;

pub fn readable_size(size: u64) -> String {
    let mut size = size as f64;
    let mut unit = "B";
    let units = ["B", "KB", "MB", "GB", "TB"];
    for &u in &units[1..] {
        size /= 1024.0;
        if size < 1024.0 {
            unit = u;
            break;
        }
    }
    format!("{:.2} {}", size, unit)
}

pub fn readable_time(time: Option<SystemTime>) -> String {
    if time.is_none() {
        return "-".to_string();
    }
    let datetime: chrono::DateTime<chrono::Local> = chrono::DateTime::from(time.unwrap());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}