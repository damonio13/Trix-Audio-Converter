//! Playlist file parsing (M3U, PLS, XSPF)
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// Generates playlist files in various formats (M3U, PLS, XSPF, ASX).
pub struct PlaylistGenerator;

impl PlaylistGenerator {
    /// Generates an M3U playlist file from a list of audio files.
    pub fn generate_m3u(files: &[String], output: &str) -> Result<(), String> {
        let mut content = "#EXTM3U\n".to_string();
        for file in files {
            let name = std::path::Path::new(file)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            content += &format!("#EXTINF:-1,{}\n{}\n", name, file);
        }
        std::fs::write(output, content).map_err(|e| e.to_string())
    }

    /// Generates a PLS playlist file from a list of audio files.
    pub fn generate_pls(files: &[String], output: &str) -> Result<(), String> {
        let mut content = "[playlist]\n".to_string();
        for (i, file) in files.iter().enumerate() {
            content += &format!("File{}={}\n", i + 1, file);
            let name = std::path::Path::new(file)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            content += &format!("Title{}={}\n", i + 1, name);
            content += &format!("Length{}=-1\n", i + 1);
        }
        content += &format!("NumberOfEntries={}\n", files.len());
        content += "Version=2\n";
        std::fs::write(output, content).map_err(|e| e.to_string())
    }

    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// Generates an ASX (WAX) playlist file from a list of audio files.
    pub fn generate_wax(files: &[String], output: &str, title: &str) -> Result<(), String> {
        let safe_title = Self::xml_escape(title);
        let mut content = format!("<ASX VERSION=\"3.0\">\n  <ENTRY>\n    <TITLE>{}</TITLE>\n", safe_title);
        for file in files {
            let path = std::path::Path::new(file)
                .to_string_lossy()
                .replace('\\', "/");
            content += &format!("    <REF HREF=\"{}\" />\n", Self::xml_escape(&path));
        }
        content += "  </ENTRY>\n</ASX>\n";
        std::fs::write(output, content).map_err(|e| e.to_string())
    }

    /// Returns all supported playlist formats as (extension, display_name) pairs.
    pub fn get_formats() -> &'static [(&'static str, &'static str)] {
        static STATIC: LazyLock<&[(&str, &str)]> = LazyLock::new(|| {
            &[
                ("m3u", "M3U (Winamp)"),
                ("pls", "PLS (foobar2000)"),
                ("wax", "WAX (Windows Media)"),
            ]
        });
        &*STATIC
    }

    /// Scans older for audio files and writes a playlist file in the requested format.
    pub fn create_folder_playlist(folder: &str, format: &str) -> Result<String, String> {
        let mut files: Vec<String> = crate::utils::list_audio_files(std::path::Path::new(folder))
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        files.sort();

        let playlist_path = format!(
            "{}/playlist.{}",
            folder,
            format
        );

        match format {
            "m3u" => Self::generate_m3u(&files, &playlist_path)?,
            "pls" => Self::generate_pls(&files, &playlist_path)?,
            "wax" => Self::generate_wax(&files, &playlist_path, "AudioMaster Playlist")?,
            _ => return Err("Formato desconhecido".into()),
        }

        Ok(playlist_path)
    }
}
