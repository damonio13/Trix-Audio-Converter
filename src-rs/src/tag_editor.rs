//! Tag Editor
//!
//! Provides tag reading/writing for audio files including ID3v1/v2,
//! Vorbis comments, and other metadata formats.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

/// Audio file tag information for reading and writing metadata.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct TagInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub track: String,
    pub disc: String,
    pub year: String,
    pub genre: String,
    pub comment: String,
    pub composer: String,
    pub copyright: String,
    pub artwork: Option<String>,
}

impl TagInfo {
    /// Creates TagInfo from raw FFmpeg metadata output.
    pub fn from_metadata(meta: &crate::metadata::AudioMetadata) -> Self {
        Self {
            title: meta.title.clone(),
            artist: meta.artist.clone(),
            album: meta.album.clone(),
            album_artist: meta.album_artist.clone(),
            track: meta.track.clone(),
            disc: String::new(),
            year: meta.date.clone(),
            genre: meta.genre.clone(),
            comment: meta.comment.clone(),
            composer: String::new(),
            copyright: String::new(),
            artwork: None,
        }
    }

    /// Reads audio metadata tags from a file using FFmpeg.
    pub fn read_tags(path: &str) -> Result<Self, String> {
        let data = crate::utils::ffprobe_json(path)?;

        let tags = data.get("format").and_then(|f| f.get("tags")).unwrap_or(&serde_json::Value::Null);

        Ok(Self {
            title: Self::get_tag(&tags, &["title", "TITLE"]),
            artist: Self::get_tag(&tags, &["artist", "ARTIST"]),
            album: Self::get_tag(&tags, &["album", "ALBUM"]),
            album_artist: Self::get_tag(&tags, &["album_artist", "ALBUMARTIST"]),
            track: Self::get_tag(&tags, &["track", "TRACK", "track_number"]),
            disc: Self::get_tag(&tags, &["disc", "DISC"]),
            year: Self::get_tag(&tags, &["date", "DATE", "year"]),
            genre: Self::get_tag(&tags, &["genre", "GENRE"]),
            comment: Self::get_tag(&tags, &["comment", "COMMENT"]),
            composer: Self::get_tag(&tags, &["composer", "COMPOSER"]),
            copyright: Self::get_tag(&tags, &["copyright", "COPYRIGHT"]),
            artwork: None,
        })
    }

    fn get_tag(tags: &serde_json::Value, keys: &[&str]) -> String {
        for key in keys {
            if let Some(val) = tags.get(*key) {
                match val {
                    serde_json::Value::String(s) => return s.clone(),
                    serde_json::Value::Array(arr) => {
                        if let Some(serde_json::Value::String(s)) = arr.first() {
                            return s.clone();
                        }
                    }
                    _ => {}
                }
            }
        }
        String::new()
    }

    /// Writes metadata tags to an audio file using FFmpeg.
    pub fn write_tags(path: &str, tags: &Self) -> Result<(), String> {
        if !crate::utils::is_safe_path(path) {
            return Err("Invalid input path".into());
        }

        let mut args: Vec<String> = vec![
            "-y".into(),
            "-i".into(),
            path.to_string(),
        ];

        let tag_pairs = [
            ("title", &tags.title),
            ("artist", &tags.artist),
            ("album", &tags.album),
            ("album_artist", &tags.album_artist),
            ("track", &tags.track),
            ("disc", &tags.disc),
            ("date", &tags.year),
            ("genre", &tags.genre),
            ("comment", &tags.comment),
            ("composer", &tags.composer),
            ("copyright", &tags.copyright),
        ];
        for (key, val) in &tag_pairs {
            if !val.is_empty() {
                args.extend(["-metadata".into(), format!("{}={}", key, val)]);
            }
        }

        if let Some(artwork_path) = &tags.artwork {
            if !crate::utils::is_safe_path(artwork_path)
                || artwork_path.contains('/') || artwork_path.contains('\\')
            {
                return Err("Invalid artwork path".into());
            }
            args.extend([
                "-i".into(),
                artwork_path.clone(),
                "-map".into(), "0".into(),
                "-map".into(), "1".into(),
                "-c:v".into(), "mjpeg".into(),
                "-metadata:s:v".into(), "title=Album cover".into(),
                "-metadata:s:v".into(), "comment=Cover (front)".into(),
            ]);
        }

        let temp_path = format!("{}.tmp{}", path, std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or(""));

        args.push("--".into());
        args.push(temp_path.clone());

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let out = crate::utils::run_ffmpeg_raw(&args_refs)
            .map_err(|e| format!("ffmpeg failed: {}", e))?;

        if out.status.success() {
            // Atomic replace: remove target first (required on Windows), then rename.
            let _ = std::fs::remove_file(path);
            std::fs::rename(&temp_path, path)
                .map_err(|e| format!("Failed to replace file: {}", e))?;
            Ok(())
        } else {
            let _ = std::fs::remove_file(&temp_path);
            Err("ffmpeg tag write failed".into())
        }
    }
}
