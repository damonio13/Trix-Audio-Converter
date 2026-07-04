//! Audio Metadata (ID3/Vorbis)
//!
//! Reading and writing of audio metadata tags including ID3v1/v2 (MP3)
//! and Vorbis comments (FLAC/OGG). Uses FFmpeg for tag manipulation.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Audio file metadata including technical properties and tag fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub filename: String,
    pub format: String,
    pub duration: f64,
    pub size: u64,
    pub bitrate: u64,
    pub codec: String,
    pub sample_rate: u64,
    pub channels: u32,
    pub bits_per_sample: u32,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub genre: String,
    pub date: String,
    pub track: String,
    pub comment: String,
    pub all_tags: HashMap<String, String>,
}

fn merge_tags(target: &mut std::collections::HashMap<String, String>, source: &serde_json::Value) {
    if let Some(obj) = source.as_object() {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                target.insert(k.clone(), s.to_string());
            }
        }
    }
}

impl AudioMetadata {
    /// Reads audio metadata from a file using FFprobe.
    pub fn read(file_path: &str) -> Result<Self, String> {
        let data = crate::utils::ffprobe_json(file_path)?;

        let fmt = data.get("format").unwrap_or(&serde_json::Value::Null);
        let streams = data.get("streams").unwrap_or(&serde_json::Value::Null);
        let audio_stream = streams.as_array()
            .and_then(|arr| arr.iter().find(|s| s.get("codec_type").and_then(|v| v.as_str()) == Some("audio")))
            .unwrap_or(&serde_json::Value::Null);

        let tags = fmt.get("tags").unwrap_or(&serde_json::Value::Null);
        let stream_tags = audio_stream.get("tags").unwrap_or(&serde_json::Value::Null);

        let mut all_tags = std::collections::HashMap::new();
        merge_tags(&mut all_tags, &tags);
        merge_tags(&mut all_tags, &stream_tags);

        let probe = crate::utils::parse_audio_probe(&data)?;
        let size = fmt.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
        let format_name = fmt.get("format_long_name").and_then(|v| v.as_str()).unwrap_or("").to_string();

        Ok(Self {
            filename: std::path::Path::new(file_path).file_name().unwrap_or_default().to_string_lossy().to_string(),
            format: format_name,
            duration: probe.duration,
            size,
            bitrate: probe.bitrate,
            codec: probe.codec,
            sample_rate: probe.sample_rate as u64,
            channels: probe.channels,
            bits_per_sample: probe.bit_depth,
            title: all_tags.get("title").cloned().unwrap_or_default(),
            artist: all_tags.get("artist").cloned().unwrap_or_default(),
            album: all_tags.get("album").cloned().unwrap_or_default(),
            album_artist: all_tags.get("album_artist").cloned().unwrap_or_default(),
            genre: all_tags.get("genre").cloned().unwrap_or_default(),
            date: all_tags.get("date").or_else(|| all_tags.get("year")).cloned().unwrap_or_default(),
            track: all_tags.get("track").cloned().unwrap_or_default(),
            comment: all_tags.get("comment").cloned().unwrap_or_default(),
            all_tags,
        })
    }

    /// Writes metadata tags to an audio file using FFmpeg.
    pub fn write_tags(file_path: &str, tags: &HashMap<String, String>, output_path: Option<&str>) -> Result<(), String> {
        if !crate::utils::is_safe_path(file_path) {
            return Err("Invalid input path".into());
        }
        if let Some(target) = output_path {
            if !crate::utils::is_safe_path(target) {
                return Err("Invalid output path".into());
            }
        }

        let mut metadata_args = Vec::new();
        for (key, value) in tags {
            if !value.is_empty() {
                metadata_args.push(format!("{}={}", key, value));
            }
        }

        let target = output_path.unwrap_or(file_path);
        let mut args: Vec<&str> = vec!["-y", "-hide_banner", "-loglevel", "error", "-i", file_path];
        for m in &metadata_args {
            args.extend(["-metadata", m]);
        }
        args.extend(["-c", "copy", "--", target]);

        let out = crate::utils::run_ffmpeg_raw(&args)?;
        if out.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).chars().take(200).collect())
        }
    }

    /// Returns the duration of an audio file in seconds.
    pub fn get_duration(file_path: &str) -> f64 {
        crate::utils::ffprobe_duration(file_path).unwrap_or(0.0)
    }
}
