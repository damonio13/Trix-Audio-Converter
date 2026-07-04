//! Output folder structure generation from metadata patterns.
//!
//! Generates hierarchical output directory trees using metadata
//! placeholders such as artist, album, year, and track number.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use std::path::Path;

/// A folder structure pattern with placeholder syntax and description.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FolderPattern {
    pub pattern: String,
    pub description: String,
}

/// Generates output folder structures from metadata patterns.
pub struct FolderStructure;

impl FolderStructure {
    /// Returns all available folder structure patterns.
    pub fn get_patterns() -> Vec<FolderPattern> {
        vec![
            FolderPattern {
                pattern: "{artist}/{album}".into(),
                description: "Artista/Album".into(),
            },
            FolderPattern {
                pattern: "{artist}/{album}/{title}".into(),
                description: "Artista/Album/Titulo".into(),
            },
            FolderPattern {
                pattern: "{artist}/{year} - {album}".into(),
                description: "Artista/Ano - Album".into(),
            },
            FolderPattern {
                pattern: "{genre}/{artist}/{album}".into(),
                description: "Genero/Artista/Album".into(),
            },
            FolderPattern {
                pattern: "{format}/{artist}/{album}".into(),
                description: "Formato/Artista/Album".into(),
            },
            FolderPattern {
                pattern: "{artist}/{album} [{year}]".into(),
                description: "Artista/Album [Ano]".into(),
            },
            FolderPattern {
                pattern: "{artist}/{year}/{album}".into(),
                description: "Artista/Ano/Album".into(),
            },
            FolderPattern {
                pattern: "{album}".into(),
                description: "Album".into(),
            },
            FolderPattern {
                pattern: "{artist}".into(),
                description: "Artista".into(),
            },
        ]
    }

    /// Builds a full output path by replacing metadata placeholders in the pattern.
    pub fn build_path(
        pattern: &str,
        base_dir: &str,
        metadata: &std::collections::HashMap<String, String>,
    ) -> String {
        let mut path = pattern.to_string();

        for (key, value) in metadata {
            let placeholder = format!("{{{}}}", key);
            let safe_name = crate::utils::sanitize_filename(value);
            path = path.replace(&placeholder, &safe_name);
        }

        // Clean up empty segments
        let segments: Vec<&str> = path.split('/')
            .filter(|s| !s.is_empty() && *s != "Unknown" && *s != "unknown")
            .collect();

        let clean_path = segments.join("/");
        let full_path = format!("{}/{}", base_dir.trim_end_matches('/'), clean_path);

        full_path
    }

    /// Ensures the parent directory of the given path exists, creating it if needed.
    pub fn ensure_parent_dir(path: &str) -> Result<(), String> {
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
        }
        Ok(())
    }

    /// Generates the full output file path for an input file using the pattern and metadata.
    pub fn get_output_path(
        input_path: &str,
        output_dir: &str,
        pattern: &str,
        format_ext: &str,
        index: usize,
        metadata: &std::collections::HashMap<String, String>,
    ) -> String {
        let filename = Path::new(input_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let mut name = Self::build_path(pattern, output_dir, metadata);

        // If pattern doesn't produce a filename, add one
        if !name.contains('.') {
            let safe_name = crate::utils::sanitize_filename(filename);
            name = format!("{}/{}.{}", name, safe_name, format_ext);
        }

        // Add index if needed
        if name.contains("{n}") {
            name = name.replace("{n}", &format!("{:03}", index));
        }

        name
    }

    /// Previews the folder structure that would be generated for a list of files.
    pub fn preview_structure(
        pattern: &str,
        files: &[String],
        metadata: &[std::collections::HashMap<String, String>],
    ) -> Vec<String> {
        files.iter().enumerate().map(|(i, file)| {
            let meta = metadata.get(i).cloned().unwrap_or_default();
            let stem = Path::new(file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("file");
            let ext = Path::new(file)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("mp3");

            let mut path = pattern.to_string();
            for (key, value) in &meta {
                let placeholder = format!("{{{}}}", key);
                path = path.replace(&placeholder, &crate::utils::sanitize_filename(value));
            }

            if !path.contains('.') {
                path = format!("{}/{}.{}", path, crate::utils::sanitize_filename(stem), ext);
            }

            path = path.replace("{n}", &format!("{:03}", i + 1));
            path
        }).collect()
    }
}
