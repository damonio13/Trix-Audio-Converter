//! Portable mode detection and data path management
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::path::PathBuf;

/// Detects if running in portable mode and provides data paths.
///
/// Portable mode is activated when:
/// 1. A `portable.txt` file exists next to the executable, OR
/// 2. A `data/` directory exists next to the executable
///
/// In portable mode, all data (logs, queue, plugins) is stored in `data/`
/// next to the executable instead of the system AppData directory.
pub struct Portable;

impl Portable {
    /// Check if the app is running in portable mode
    pub fn is_portable() -> bool {
        let exe_dir = Self::exe_dir();
        exe_dir.join("portable.txt").exists() || exe_dir.join("data").is_dir()
    }

    /// Get the directory where the executable is located
    pub fn exe_dir() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Get the data directory for storing app data
    /// - Portable mode: `<exe_dir>/data/`
    /// - Normal mode: `<AppData>/trix-audio-converter/`
    pub fn data_dir() -> PathBuf {
        if Self::is_portable() {
            let dir = Self::exe_dir().join("data");
            let _ = std::fs::create_dir_all(&dir);
            dir
        } else {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("trix-audio-converter")
        }
    }

    /// Get the logs directory
    pub fn logs_dir() -> PathBuf {
        let dir = Self::data_dir().join("logs");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Get the plugins directory
    pub fn plugins_dir() -> PathBuf {
        let dir = Self::data_dir().join("plugins");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Get the queue state file path
    pub fn queue_path() -> PathBuf {
        Self::data_dir().join("queue_state.json")
    }

    /// Get the settings file path
    pub fn settings_path() -> PathBuf {
        Self::data_dir().join("settings.json")
    }
}
