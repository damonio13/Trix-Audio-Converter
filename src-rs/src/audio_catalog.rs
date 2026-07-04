//! Audio file cataloging and library management.
//!
//! Maintains a searchable database of audio files with metadata indexing,
//! duplicate detection, and smart playlist generation.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};

fn str_contains_ci(haystack: &str, needle_lower: &str) -> bool {
    let bytes = haystack.as_bytes();
    if bytes.len() < 512 {
        let mut buf = [0u8; 512];
        let len = bytes.len().min(buf.len());
        for i in 0..len {
            buf[i] = bytes[i].to_ascii_lowercase();
        }
        if let Ok(s) = std::str::from_utf8(&buf[..len]) {
            return s.contains(needle_lower);
        }
    }
    haystack.to_lowercase().contains(needle_lower)
}

fn str_eq_ci(a: &str, b_lower: &str) -> bool {
    let bytes = a.as_bytes();
    if bytes.len() < 512 {
        let mut buf = [0u8; 512];
        let len = bytes.len().min(buf.len());
        for i in 0..len {
            buf[i] = bytes[i].to_ascii_lowercase();
        }
        if let Ok(s) = std::str::from_utf8(&buf[..len]) {
            return s == b_lower;
        }
    }
    a.to_lowercase() == b_lower
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A single entry in the audio file catalog.
pub struct AudioCatalogEntry {
    pub id: String,
    pub file_path: String,
    pub filename: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
    pub format: String,
    pub bitrate: u32,
    pub sample_rate: u32,
    pub channels: u32,
    pub fingerprint: String,
    pub file_size: u64,
    pub date_added: String,
    pub tags: Vec<String>,
}

/// Manages a searchable catalog of audio files with metadata indexing.
pub struct AudioCatalog {
    collection: crate::utils::JsonCollection<AudioCatalogEntry>,
}

impl AudioCatalog {
    /// Creates a new audio catalog backed by a JSON collection.
    pub fn new() -> Self {
        let catalog_dir = crate::portable::Portable::data_dir().join("catalog");
        let _ = std::fs::create_dir_all(&catalog_dir);

        Self {
            collection: crate::utils::JsonCollection::new(
                catalog_dir.join("index.json"),
                10 * 1024 * 1024,
            ),
        }
    }

    /// Loads the catalog from disk.
    pub fn load(&mut self) {
        self.collection.load();
    }

    /// Saves the catalog to disk.
    pub fn save(&self) -> Result<(), String> {
        self.collection.save()
    }

    /// Adds a single audio file to the catalog, extracting its metadata.
    pub fn add_file(&mut self, file_path: &str) -> Result<AudioCatalogEntry, String> {
        let metadata = crate::metadata::AudioMetadata::read(file_path)?;
        let analysis = crate::audio_analyzer::AudioAnalyzer::analyze(file_path)?;

        let entry = AudioCatalogEntry {
            id: uuid::Uuid::new_v4().to_string(),
            file_path: file_path.to_string(),
            filename: std::path::Path::new(file_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            title: metadata.title,
            artist: metadata.artist,
            album: metadata.album,
            duration: analysis.duration,
            format: analysis.codec,
            bitrate: analysis.bitrate as u32,
            sample_rate: analysis.sample_rate,
            channels: analysis.channels,
            fingerprint: String::new(),
            file_size: analysis.file_size,
            date_added: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            tags: Vec::new(),
        };

        self.collection.push(entry.clone());
        let _ = self.save();
        Ok(entry)
    }

    /// Adds all audio files from a directory to the catalog.
    pub fn add_directory(&mut self, dir_path: &str, recursive: bool) -> Result<Vec<AudioCatalogEntry>, String> {
        let mut entries = Vec::new();

        for path in crate::utils::list_audio_files(std::path::Path::new(dir_path)) {
            match self.add_file(&path.to_string_lossy()) {
                Ok(e) => entries.push(e),
                Err(e) => eprintln!("Erro: {}", e),
            }
        }

        if recursive {
            if let Ok(read_dir) = std::fs::read_dir(dir_path) {
                for entry in read_dir.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(sub_entries) = self.add_directory(&path.to_string_lossy(), true) {
                            entries.extend(sub_entries);
                        }
                    }
                }
            }
        }

        Ok(entries)
    }

    /// Searches the catalog by title, artist, album, filename, or tags.
    pub fn search(&self, query: &str) -> Vec<&AudioCatalogEntry> {
        let q = query.to_lowercase();
        self.collection.items().iter()
            .filter(|e| {
                str_contains_ci(&e.title, &q)
                    || str_contains_ci(&e.artist, &q)
                    || str_contains_ci(&e.album, &q)
                    || str_contains_ci(&e.filename, &q)
                    || e.tags.iter().any(|t| str_contains_ci(t, &q))
            })
            .collect()
    }

    /// Searches the catalog by a specific field (title, artist, album, format).
    pub fn search_by_field(&self, field: &str, value: &str) -> Vec<&AudioCatalogEntry> {
        let v = value.to_lowercase();
        self.collection.items().iter()
            .filter(|e| {
                match field {
                    "title" => str_contains_ci(&e.title, &v),
                    "artist" => str_contains_ci(&e.artist, &v),
                    "album" => str_contains_ci(&e.album, &v),
                    "format" => str_eq_ci(&e.format, &v),
                    _ => false,
                }
            })
            .collect()
    }

    /// Returns aggregate statistics about the catalog.
    pub fn get_stats(&self) -> CatalogStats {
        let entries = self.collection.items();
        let total_files = entries.len();
        let total_duration: f64 = entries.iter().map(|e| e.duration).sum();
        let total_size: u64 = entries.iter().map(|e| e.file_size).sum();
        let formats: std::collections::HashMap<String, usize> = entries.iter()
            .fold(std::collections::HashMap::new(), |mut acc, e| {
                *acc.entry(e.format.clone()).or_insert(0) += 1;
                acc
            });

        CatalogStats {
            total_files,
            total_duration,
            total_size,
            formats,
        }
    }

    /// Exports the catalog to a CSV file.
    pub fn export_csv(&self, output: &str) -> Result<(), String> {
        fn csv_escape(s: &str) -> String {
            if s.starts_with(['=', '+', '-', '@', '\t', '\r', '\n']) {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else if s.contains(',') || s.contains('"') || s.contains('\n') {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.to_string()
            }
        }
        let mut csv = "ID,Filename,Title,Artist,Album,Duration,Format,Size\n".to_string();
        for entry in self.collection.items() {
            csv += &format!("{},{},{},{},{},{:.1},{},{}\n",
                entry.id, csv_escape(&entry.filename), csv_escape(&entry.title),
                csv_escape(&entry.artist), csv_escape(&entry.album),
                entry.duration, entry.format, entry.file_size
            );
        }
        std::fs::write(output, csv).map_err(|e| e.to_string())
    }

    /// Returns all entries in the catalog.
    pub fn get_entries(&self) -> &[AudioCatalogEntry] {
        self.collection.items()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Aggregate statistics for the audio catalog.
pub struct CatalogStats {
    pub total_files: usize,
    pub total_duration: f64,
    pub total_size: u64,
    pub formats: std::collections::HashMap<String, usize>,
}
