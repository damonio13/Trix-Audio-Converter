//! Batch file renaming with configurable patterns.
//!
//! Renames multiple audio files at once using template-based rules,
//! tag substitution, and sequential numbering schemes.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// Batch file renaming engine with pattern-based name generation.
pub struct BatchRenamer;

impl BatchRenamer {
    fn sanitize_segment(s: &str) -> String {
        crate::utils::sanitize_filename(s)
    }

    fn sanitize_pattern_value(s: &str) -> String {
        s.split('/')
            .map(|seg| Self::sanitize_segment(seg))
            .collect::<Vec<_>>()
            .join("/")
    }

    fn validate_output_path(path: &str) -> Result<(), String> {
        if !crate::utils::is_safe_path(path) {
            return Err("Caminho de saida invalido".into());
        }
        Ok(())
    }

    /// Renames files according to a pattern, returning old and new paths.
    pub fn rename_files(
        files: &[String],
        pattern: &str,
        output_dir: &str,
    ) -> Result<Vec<(String, String)>, String> {
        let mut renames = Vec::new();

        for file in files {
            let metadata = crate::metadata::AudioMetadata::read(file)?;
            let path = std::path::Path::new(file);
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("mp3");

            let new_name = Self::apply_pattern(pattern, &metadata, ext);

            let new_path = if output_dir.is_empty() {
                path.parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join(&new_name)
                    .to_string_lossy()
                    .to_string()
            } else {
                format!("{}/{}", output_dir, new_name)
            };

            Self::validate_output_path(&new_path)?;
            renames.push((file.clone(), new_path));
        }

        Ok(renames)
    }

    /// Applies a naming pattern template using audio metadata placeholders.
    pub fn apply_pattern(pattern: &str, metadata: &crate::metadata::AudioMetadata, ext: &str) -> String {
        let track_num = metadata.track.parse::<u32>().unwrap_or(0);
        let track = if track_num > 0 { format!("{:02}", track_num) } else { String::new() };
        let mut result = String::with_capacity(pattern.len() + 64);
        result.push_str(pattern);
        let replacements = [
            ("{artist}", Self::sanitize_pattern_value(&metadata.artist)),
            ("{title}", Self::sanitize_pattern_value(&metadata.title)),
            ("{album}", Self::sanitize_pattern_value(&metadata.album)),
            ("{track}", metadata.track.to_string()),
            ("{genre}", Self::sanitize_pattern_value(&metadata.genre)),
            ("{ext}", ext.to_string()),
            ("{track:02}", track),
        ];
        for (placeholder, value) in &replacements {
            if result.contains(placeholder) {
                result = result.replace(placeholder, value);
            }
        }
        result
    }

    /// Previews the rename results without executing them.
    pub fn preview_renames(
        files: &[String],
        pattern: &str,
    ) -> Result<Vec<(String, String)>, String> {
        let mut results = Vec::new();

        for file in files {
            let metadata = crate::metadata::AudioMetadata::read(file)?;
            let path = std::path::Path::new(file);
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("mp3");

            let new_name = Self::apply_pattern(pattern, &metadata, ext);
            let old_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

            results.push((old_name, new_name));
        }

        Ok(results)
    }

    /// Executes a list of rename operations on disk.
    pub fn execute_renames(renames: &[(String, String)]) -> Result<Vec<String>, String> {
        let mut renamed = Vec::new();

        for (old, new) in renames {
            if let Err(e) = std::fs::rename(old, new) {
                return Err(format!("Erro ao renomear {}: {}", old, e));
            }
            renamed.push(new.clone());
        }

        Ok(renamed)
    }

    /// Returns the list of predefined naming patterns.
    pub fn get_patterns() -> &'static [(&'static str, &'static str)] {
        static STATIC: LazyLock<&[(&'static str, &'static str)]> = LazyLock::new(|| &[
            ("{artist} - {title}", "Artista - Titulo"),
            ("{track:02} - {artist} - {title}", "01 - Artista - Titulo"),
            ("{artist} - {album} - {track:02} - {title}", "Artista - Album - 01 - Titulo"),
            ("{title}", "Apenas Titulo"),
            ("{artist}/{album}/{track:02} - {title}", "Artista/Album/01 - Titulo"),
            ("{artist} ({year}) - {title}", "Artista (2024) - Titulo"),
            ("{genre}/{artist} - {title}", "Genero/Artista - Titulo"),
        ]);
        &*STATIC
    }
}
