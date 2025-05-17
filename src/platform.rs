use std::path::PathBuf;

pub struct Platform;

// Platform Specific Getters/Functions
impl Platform {
    // Returns the home directory of the current user or the root directory
    pub fn home_dir() -> PathBuf {
        #[cfg(target_os = "windows")]
        let default = PathBuf::from("C:\\");
        #[cfg(target_os = "linux")] 
        let default = PathBuf::from("/");
        #[cfg(target_os = "macos")]
        let default = PathBuf::from("/");

        dirs::home_dir().unwrap_or(default)
    }

    pub fn special_dirs() -> Vec<PathBuf> {
        let mut dirs = vec![];

        dirs.push(dirs::desktop_dir());
        dirs.push(dirs::download_dir());
        dirs.push(dirs::document_dir());
        dirs.push(dirs::picture_dir());
        dirs.push(dirs::video_dir());
        dirs.push(dirs::audio_dir());

        dirs.iter().filter(|d| d.is_some()).map(|d| d.clone().unwrap()).collect::<Vec<_>>()
    }
}