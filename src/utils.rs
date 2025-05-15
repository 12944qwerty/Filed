use std::{path::{Path, PathBuf}, time::SystemTime};

use chrono::{DateTime, Local, Duration};

use crate::views::explorer::FileType;

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

    let datetime: DateTime<Local> = DateTime::from(time.unwrap());
    let now: DateTime<Local> = Local::now();

    // If the datetime is within the last hour
    if now.signed_duration_since(datetime).num_minutes() < 60 {
        let duration = now.signed_duration_since(datetime);
        if duration.num_seconds() < 60 {
            return format!("{} seconds ago", duration.num_seconds());
        } else if duration.num_minutes() < 60 {
            return format!("{} minutes ago", duration.num_minutes());
        }
    }

    // If the date is today
    if datetime.date_naive() == now.date_naive() {
        return format!("Today at {}", datetime.format("%I:%M %p")); // 12-hour format with AM/PM
    }

    // If the date is yesterday
    if datetime.date_naive() == now.date_naive() - Duration::days(1) {
        return format!("Yesterday at {}", datetime.format("%I:%M %p"));
    }

    // Otherwise, show the full date and time
    datetime.format("%m/%d/%Y %I:%M %p").to_string()
}

pub fn image_from_type(file_type: FileType) -> String {
    match file_type {
        FileType::Directory => "resources/folder.ico",
        FileType::File => "resources/file.ico",
        FileType::Image => "resources/image.ico",
        FileType::Video => "resources/video.ico",
        FileType::Audio => "resources/audio.ico",
        FileType::Document => "resources/document.ico",
        FileType::Unknown => "resources/unknown.ico",
    }.to_owned()
}

pub fn file_type_from_extension(ext: &str) -> FileType {
    let image_extensions = vec!["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "svg"];
    let video_extensions = vec!["mp4", "avi", "mkv", "mov", "flv", "wmv", "webm", "mpeg"];
    let audio_extensions = vec!["mp3", "wav", "aac", "flac", "ogg", "m4a"];
    let document_extensions = vec!["pdf", "docx", "doc", "odt", "pptx", "xlsx", "rtf", "epub", "html"];
    let misc_extensions = vec!["txt", "md", ""];

    match ext {
        ext if image_extensions.contains(&ext) => FileType::Image,
        ext if video_extensions.contains(&ext) => FileType::Video,
        ext if audio_extensions.contains(&ext) => FileType::Audio,
        ext if document_extensions.contains(&ext) => FileType::Document,
        ext if misc_extensions.contains(&ext) => FileType::File,
        _ => FileType::Unknown,
    }
}