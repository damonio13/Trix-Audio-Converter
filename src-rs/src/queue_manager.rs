//! Queue Manager
//!
//! Manages the conversion queue state, including persistence between
//! sessions and settings for batch processing.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Conversion queue settings for batch processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSettings {
    pub format_key: String,
    pub sample_rate: String,
    pub channels: String,
    pub bitrate: String,
    pub volume: i32,
    pub codec_copy: bool,
    pub trim_start: f64,
    pub trim_end: f64,
    pub output_directory: String,
    pub output_in_same_folder: bool,
    pub output_suffix: String,
    pub quality: u8,
    pub output_pattern: String,
    pub output_subfolder: bool,
    pub max_output_size_mb: f64,
}

impl Default for QueueSettings {
    fn default() -> Self {
        Self {
            format_key: ".mp3".into(),
            sample_rate: "Original".into(),
            channels: "Original".into(),
            bitrate: "192".into(),
            volume: 0,
            codec_copy: false,
            trim_start: 0.0,
            trim_end: 0.0,
            output_directory: String::new(),
            output_in_same_folder: true,
            output_suffix: "_trix".into(),
            quality: 100,
            output_pattern: String::new(),
            output_subfolder: false,
            max_output_size_mb: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Persisted conversion queue with folders and settings.
pub struct QueueData {
    pub folders: Vec<String>,
    pub settings: QueueSettings,
}

#[derive(Clone)]
/// Manages persistence of the conversion queue to disk.
pub struct QueueManager {
    queue_path: PathBuf,
}

impl QueueManager {
    /// Creates a new QueueManager with default settings.
    pub fn new() -> Self {
        let queue_path = crate::portable::Portable::queue_path();

        Self { queue_path }
    }

    /// Persists the current queue state to disk.
    pub fn save(&self, folders: &[String], settings: &QueueSettings) -> bool {
        let data = QueueData {
            folders: folders.to_vec(),
            settings: settings.clone(),
        };
        crate::utils::save_versioned_json(&self.queue_path, &data).is_ok()
    }

    /// Restores a previously saved queue state from disk.
    pub fn load(&self) -> Option<QueueData> {
        crate::utils::load_versioned_json(&self.queue_path, 1024 * 1024).ok()
    }

    /// Clears the persisted queue state file.
    pub fn clear(&self) {
        let _ = std::fs::remove_file(&self.queue_path);
    }

    /// Checks if a saved queue state exists on disk.
    pub fn has_saved_queue(&self) -> bool {
        self.queue_path.exists()
    }
}
