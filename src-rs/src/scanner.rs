//! File Scanner
//!
//! Scans directories for supported audio files, filtering by extension
//! and collecting file metadata for the conversion queue.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use crate::formats::INPUT_EXTENSIONS;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Maximum folder recursion depth to prevent infinite loops on symlink cycles.
const MAX_SCAN_DEPTH: usize = 20;

/// Summary of a scanned folder including file count and total size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderInfo {
    /// Absolute path to the folder.
    pub path: String,
    /// Last component of the path (folder display name).
    pub name: String,
    /// Number of supported audio files found inside (recursive).
    pub file_count: usize,
    /// Total size in bytes of all discovered audio files.
    pub total_size: u64,
    /// Human-readable version of `total_size` (e.g. `"12.3 MB"`).
    pub total_size_human: String,
}

impl FolderInfo {
    pub fn new(path: &str) -> Self {
        let p = Path::new(path);
        Self {
            path: path.to_string(),
            name: p.file_name().unwrap_or_default().to_string_lossy().to_string(),
            file_count: 0,
            total_size: 0,
            total_size_human: "0 B".into(),
        }
    }

    /// Converts `total_size` to a human-readable string and stores it in `total_size_human`.
    pub fn update_human_size(&mut self) {
        self.total_size_human = crate::formats::human_size(self.total_size);
    }
}

#[derive(Debug, Clone)]
/// Result of scanning folders for audio files.
pub struct ScanResult {
    pub files: Vec<(String, String)>,
    pub folder_infos: Vec<FolderInfo>,
}

/// Recursively scans directories for supported audio files.
pub struct FileScanner;

impl FileScanner {
    /// Scans multiple folders for supported audio files and returns file paths.
    pub fn scan_folders(folders: &[String], output_ext: &str) -> Vec<(String, String)> {
        let mut files = Vec::new();

        for folder_path in folders {
            let p = Path::new(folder_path);
            if !p.is_dir() {
                continue;
            }
            let mut visited = HashSet::new();
            Self::scan_recursive(p, output_ext, &mut files, 0, &mut visited, None);
        }

        files
    }

    /// Scans folders and returns detailed info including file sizes and counts.
    pub fn scan_folders_info(folders: &[String], output_ext: &str) -> ScanResult {
        let mut folder_infos = Vec::new();
        let mut all_files = Vec::new();
        let mut seen_hashes = std::collections::HashSet::new();
        let mut visited_dirs = HashSet::new();

        for folder_path in folders {
            let mut info = FolderInfo::new(folder_path);
            let p = Path::new(folder_path);

            if !p.is_dir() {
                folder_infos.push(info);
                continue;
            }

            Self::scan_recursive(p, output_ext, &mut all_files, 0, &mut visited_dirs, Some((&mut info, &mut seen_hashes)));
            info.update_human_size();
            folder_infos.push(info);
        }

        ScanResult {
            files: all_files,
            folder_infos,
        }
    }

    /// Recursively traverses `dir` up to `MAX_SCAN_DEPTH` levels.
    ///
    /// Uses a `visited` set of canonical paths to guard against symlink cycles.
    /// When `info_and_hashes` is `Some`, also accumulates `FolderInfo` stats
    /// and deduplicates files by size+mtime fingerprint.
    fn scan_recursive(
        dir: &Path,
        output_ext: &str,
        files: &mut Vec<(String, String)>,
        depth: usize,
        visited: &mut HashSet<PathBuf>,
        mut info_and_hashes: Option<(&mut FolderInfo, &mut std::collections::HashSet<String>)>,
    ) {
        if depth > MAX_SCAN_DEPTH {
            return;
        }

        let canonical = match std::fs::canonicalize(dir) {
            Ok(c) => c,
            Err(_) => return,
        };
        if !visited.insert(canonical) {
            return;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::scan_recursive(&path, output_ext, files, depth + 1, visited, info_and_hashes.as_mut().map(|(i, h)| (&mut **i, &mut **h)));
            } else if let Some(ext) = path.extension() {
                let ext_lossy = ext.to_string_lossy();
                let ext_lower = ext_lossy.to_lowercase();
                let mut dotted = [0u8; 256];
                dotted[0] = b'.';
                let bytes = ext_lower.as_bytes();
                let copy_len = bytes.len().min(dotted.len() - 1);
                dotted[1..=copy_len].copy_from_slice(&bytes[..copy_len]);
                let ext_key = std::str::from_utf8(&dotted[..=copy_len]).unwrap_or("");
                if INPUT_EXTENSIONS.contains(ext_key) {
                    if let Some((_, seen_hashes)) = &mut info_and_hashes {
                        let hash = file_fingerprint(&path);
                        if !hash.is_empty() && seen_hashes.contains(&hash) {
                            continue;
                        }
                        if !hash.is_empty() {
                            seen_hashes.insert(hash);
                        }
                    }

                    let mut output_file = path.clone();
                    output_file.set_extension(output_ext.trim_start_matches('.'));
                    if output_file == path {
                        continue;
                    }

                    if let Some((info, _)) = &mut info_and_hashes {
                        info.file_count += 1;
                        if let Ok(meta) = std::fs::metadata(&path) {
                            info.total_size += meta.len();
                        }
                    }

                    files.push((
                        path.to_string_lossy().to_string(),
                        output_file.to_string_lossy().to_string(),
                    ));
                }
            }
        }
    }

    /// Checks if a file extension is in the supported input formats list.
    pub fn is_audio_file(file_path: &str) -> bool {
        let p = Path::new(file_path);
        if let Some(ext) = p.extension() {
            let ext_lossy = ext.to_string_lossy();
            let ext_lower = ext_lossy.to_lowercase();
            let mut dotted = [0u8; 256];
            dotted[0] = b'.';
            let bytes = ext_lower.as_bytes();
            let copy_len = bytes.len().min(dotted.len() - 1);
            dotted[1..=copy_len].copy_from_slice(&bytes[..copy_len]);
            let ext_key = std::str::from_utf8(&dotted[..=copy_len]).unwrap_or("");
            return INPUT_EXTENSIONS.contains(ext_key);
        }
        false
    }

    /// Counts audio files recursively in a directory.
    pub fn count_files(folders: &[String], output_ext: &str) -> usize {
        Self::scan_folders(folders, output_ext).len()
    }
}

/// Builds a lightweight duplicate-detection fingerprint from the file's
/// `size` and last-modified timestamp (`mtime` in nanoseconds).
/// Returns an empty string if the metadata cannot be read.
fn file_fingerprint(path: &Path) -> String {
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return String::new(),
    };
    let size = meta.len();
    let mtime = meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}:{}", size, mtime)
}
