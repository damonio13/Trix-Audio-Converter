//! Watch folder for automatic conversion of new files
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

/// Watches directories for new audio files and triggers automatic conversion.
pub struct WatchFolder {
    watching: Arc<Mutex<bool>>,
    watched_dirs: Arc<Mutex<Vec<WatchedDir>>>,
}

/// A directory being watched for new audio files.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WatchedDir {
    pub path: PathBuf,
    pub output_format: String,
    pub output_dir: String,
    pub recursive: bool,
}

impl WatchFolder {
    /// Creates a new WatchFolder with no watched directories.
    pub fn new() -> Self {
        Self {
            watching: Arc::new(Mutex::new(false)),
            watched_dirs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds a directory to be watched for new audio files.
    pub fn add_watch(&self, dir: &str, output_format: &str, output_dir: &str, recursive: bool) -> Result<(), String> {
        if !crate::utils::is_safe_path(dir) {
            return Err("Invalid watched directory".into());
        }
        if !crate::utils::is_safe_path(output_dir) {
            return Err("Invalid output directory".into());
        }
        let mut dirs = self.watched_dirs.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        dirs.push(WatchedDir {
            path: PathBuf::from(dir),
            output_format: output_format.to_string(),
            output_dir: output_dir.to_string(),
            recursive,
        });
        Ok(())
    }

    /// Removes a directory from the watch list.
    pub fn remove_watch(&self, dir: &str) -> Result<(), String> {
        let mut dirs = self.watched_dirs.lock().map_err(|e| format!("Lock poisoned: {}", e))?;
        dirs.retain(|d| d.path.to_string_lossy() != dir);
        Ok(())
    }

    /// Spawns a background thread to watch folder directories for new audio files.
    pub fn start_watching(&self) {
        let watching = self.watching.clone();
        let dirs = self.watched_dirs.clone();

        *watching.lock().unwrap() = true;

        thread::spawn(move || {
            let mut last_state: std::collections::HashMap<PathBuf, HashSet<String>> =
                std::collections::HashMap::new();

            // Initialize state
            {
                let dirs = dirs.lock().unwrap();
                for watched in dirs.iter() {
                    if let Ok(entries) = std::fs::read_dir(&watched.path) {
                        let files: HashSet<String> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().is_file())
                            .map(|e| e.path().to_string_lossy().to_string())
                            .collect();
                        last_state.insert(watched.path.clone(), files);
                    }
                }
            }

            loop {
                if !*watching.lock().unwrap() {
                    break;
                }

                thread::sleep(std::time::Duration::from_secs(2));

                let dirs = dirs.lock().unwrap();
                for watched in dirs.iter() {
                    if let Ok(entries) = std::fs::read_dir(&watched.path) {
                        let current_files: HashSet<String> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().is_file())
                            .map(|e| e.path().to_string_lossy().to_string())
                            .collect();

                        if let Some(prev_files) = last_state.get(&watched.path) {
                            let new_files: Vec<&String> = current_files
                                .iter()
                                .filter(|f| !prev_files.contains(*f))
                                .collect();

                            for file in new_files {
                                let ext = std::path::Path::new(file)
                                    .extension()
                                    .and_then(|e| e.to_str())
                                    .unwrap_or("");

                                if matches!(ext, "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a") {
                                    let output_path = format!(
                                        "{}/{}.{}",
                                        watched.output_dir,
                                        crate::utils::file_stem(std::path::Path::new(file)),
                                        watched.output_format
                                    );

                                    if crate::utils::is_safe_path(&output_path) {
                                        let _ = std::fs::create_dir_all(&watched.output_dir);
                                        let _ = crate::utils::run_ffmpeg_raw(&[
                                            "-hide_banner", "-i", file, "-y", "--", &output_path,
                                        ]);
                                    }
                                }
                            }
                        }

                        last_state.insert(watched.path.clone(), current_files);
                    }
                }
            }
        });
    }

    /// Signals the background watch thread to stop.
    pub fn stop_watching(&self) {
        *self.watching.lock().unwrap() = false;
    }

    /// Checks if the folder watch thread is currently active.
    pub fn is_watching(&self) -> bool {
        *self.watching.lock().unwrap()
    }

    /// Returns a list of currently watched directories.
    pub fn get_watched_dirs(&self) -> Vec<WatchedDir> {
        let dirs_snapshot = {
            match self.watched_dirs.lock() {
                Ok(guard) => guard.clone(),
                Err(p) => p.into_inner().clone(),
            }
        };
        dirs_snapshot
    }
}
