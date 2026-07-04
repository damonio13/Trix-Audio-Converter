//! Automatic tag extraction from filenames using pattern matching.
//!
//! Parses audio filenames to extract artist, album, track number,
//! and title metadata using configurable regex and delimiter patterns.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Result of automatic tag extraction from a filename.
pub struct AutoTagResult {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub track: Option<u32>,
    pub year: Option<String>,
    pub genre: Option<String>,
}

/// Extracts audio metadata tags from filenames using pattern matching.
pub struct AutoTagger;

impl AutoTagger {
    /// Parses a filename to extract artist, album, title, track, and year.
    pub fn parse_filename(filename: &str) -> AutoTagResult {
        let stem = crate::utils::file_stem_or(Path::new(filename), filename);

        let mut result = AutoTagResult {
            artist: None,
            album: None,
            title: None,
            track: None,
            year: None,
            genre: None,
        };

        // Try common patterns:
        // "01 - Artist - Title"
        // "Artist - Title"
        // "Artist - Album - 01 - Title"
        // "01. Artist - Title"
        // "Title (Artist)"

        let parts: Vec<&str> = stem.split(" - ").collect();

        if parts.len() >= 3 {
            // Pattern: "Artist - Album - Title" or "01 - Artist - Title"
            if let Ok(num) = parts[0].trim().parse::<u32>() {
                result.track = Some(num);
                result.artist = Some(parts[1].trim().to_string());
                result.title = Some(parts[2..].join(" - "));
            } else {
                result.artist = Some(parts[0].trim().to_string());
                result.album = Some(parts[1].trim().to_string());
                result.title = Some(parts[2..].join(" - "));
            }
        } else if parts.len() == 2 {
            // Pattern: "Artist - Title" or "01 - Title"
            if let Ok(num) = parts[0].trim().parse::<u32>() {
                result.track = Some(num);
                result.title = Some(parts[1].trim().to_string());
            } else {
                result.artist = Some(parts[0].trim().to_string());
                result.title = Some(parts[1].trim().to_string());
            }
        } else {
            // Just the title
            result.title = Some(stem.to_string());
        }

        // Try to extract year from parentheses: "Title (2024)"
        if let Some(start) = stem.find('(') {
            if let Some(end) = stem.find(')') {
                let year_str = &stem[start + 1..end];
                if year_str.len() == 4 && year_str.chars().all(|c| c.is_ascii_digit()) {
                    result.year = Some(year_str.to_string());
                }
            }
        }

        // Try to extract track from brackets: "Title [01]"
        if let Some(start) = stem.find('[') {
            if let Some(end) = stem.find(']') {
                let track_str = &stem[start + 1..end];
                if let Ok(num) = track_str.parse::<u32>() {
                    result.track = Some(num);
                }
            }
        }

        result
    }

    /// Applies tags parsed from the filename to the file's metadata tags.
    pub fn apply_tags_from_filename(path: &str) -> Result<AutoTagResult, String> {
        let filename = Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Invalid filename")?;

        let parsed = Self::parse_filename(filename);

        // Read existing tags
        let mut tags = crate::tag_editor::TagInfo::read_tags(path)?;

        // Apply parsed tags (only if not already set)
        if tags.artist.is_empty() {
            tags.artist = parsed.artist.clone().unwrap_or_default();
        }
        if tags.album.is_empty() {
            tags.album = parsed.album.clone().unwrap_or_default();
        }
        if tags.title.is_empty() {
            tags.title = parsed.title.clone().unwrap_or_default();
        }
        if let Some(track) = &parsed.track {
            if tags.track.is_empty() {
                tags.track = track.to_string();
            }
        }
        if tags.year.is_empty() {
            tags.year = parsed.year.clone().unwrap_or_default();
        }

        crate::tag_editor::TagInfo::write_tags(path, &tags)?;

        Ok(parsed)
    }

    /// Applies filename-based tags to a batch of files.
    pub fn batch_apply_tags(paths: &[String]) -> Vec<(String, Result<AutoTagResult, String>)> {
        paths.iter()
            .map(|p| {
                let result = Self::apply_tags_from_filename(p);
                (p.clone(), result)
            })
            .collect()
    }

    /// Cleans a filename by normalizing whitespace and removing bracket characters.
    pub fn clean_filename(name: &str) -> String {
        let mut result = String::with_capacity(name.len());
        let mut prev_was_space = false;
        for ch in name.chars() {
            if ch == '[' || ch == ']' || ch == '(' || ch == ')' || ch == '{' || ch == '}' {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else if ch.is_ascii_whitespace() {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(ch);
                prev_was_space = false;
            }
        }
        result.trim().to_string()
    }
}
