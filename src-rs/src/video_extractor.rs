//! Audio extraction from video files (MP4, MKV, AVI)
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::path::Path;
use std::sync::LazyLock;

const SUPPORTED_FORMATS: &[&str] = &["mp3", "flac", "aac", "m4a", "ogg", "opus", "wav"];

fn is_valid_format(fmt: &str) -> bool {
    SUPPORTED_FORMATS.contains(&fmt)
}

fn output_ext_for_format(fmt: &str) -> &str {
    match fmt {
        "mp3" => "mp3",
        "flac" => "flac",
        "aac" | "m4a" => "m4a",
        "ogg" => "ogg",
        "opus" => "opus",
        "wav" => "wav",
        _ => "mp3",
    }
}

/// Information about a video file's streams and metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoInfo {
    pub path: String,
    pub has_video: bool,
    pub has_audio: bool,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub duration: Option<f64>,
    pub resolution: Option<String>,
}

/// Extracts audio tracks from video files.
pub struct VideoExtractor;

impl VideoExtractor {
    /// Probes a video file and returns information about its streams.
    pub fn probe(path: &str) -> Result<VideoInfo, String> {
        if !crate::utils::is_safe_path(path) {
            return Err("Caminho invalido".into());
        }

        let data = crate::utils::ffprobe_json(path)?;

        let streams = data["streams"].as_array().cloned().unwrap_or_default();
        let mut has_video = false;
        let mut has_audio = false;
        let mut video_codec = None;
        let mut audio_codec = None;
        let mut resolution = None;

        for stream in &streams {
            if let Some(codec_type) = stream["codec_type"].as_str() {
                match codec_type {
                    "video" => {
                        has_video = true;
                        video_codec = stream["codec_name"].as_str().map(|s| s.to_string());
                        let w = stream["width"].as_u64().unwrap_or(0);
                        let h = stream["height"].as_u64().unwrap_or(0);
                        if w > 0 && h > 0 {
                            resolution = Some(format!("{}x{}", w, h));
                        }
                    }
                    "audio" => {
                        has_audio = true;
                        audio_codec = stream["codec_name"].as_str().map(|s| s.to_string());
                    }
                    _ => {}
                }
            }
        }

        let duration = crate::utils::parse_audio_probe(&data).ok().map(|p| p.duration);

        Ok(VideoInfo {
            path: path.to_string(),
            has_video,
            has_audio,
            video_codec,
            audio_codec,
            duration,
            resolution,
        })
    }

    /// Extracts the audio stream from a video file using FFmpeg.
    pub fn extract_audio(input: &str, output: &str, format: &str, bitrate: Option<&str>) -> Result<String, String> {
        crate::utils::validate_input_output(input, output)?;
        if !is_valid_format(format) {
            return Err(format!("Formato invalido: '{}'. Use: {}", format, SUPPORTED_FORMATS.join(", ")));
        }

        let mut args = vec!["-i", input];

        if crate::utils::codec_for_format(format).is_some() {
            crate::utils::apply_codec_args(&mut args, format, bitrate, None)?;
        } else {
            args.extend(["-codec:a", "copy"]);
        }

        args.extend(["-vn", "-y", "--", output]);

        crate::utils::run_ffmpeg_raw_checked(&args)?;

        Ok(output.to_string())
    }

    /// Extracts audio streams from multiple video files and saves them to a destination directory.
    pub fn extract_audio_from_all(inputs: &[String], output_dir: &str, format: &str) -> Result<Vec<String>, String> {
        if !is_valid_format(format) {
            return Err(format!("Formato invalido: '{}'. Use: {}", format, SUPPORTED_FORMATS.join(", ")));
        }
        if !crate::utils::is_safe_path(output_dir) {
            return Err("Diretorio de saida invalido".into());
        }

        let ext = output_ext_for_format(format);
        let mut outputs = Vec::new();
        for input in inputs {
            if !crate::utils::is_safe_path(input) {
                continue;
            }
            let stem = crate::utils::file_stem_or(Path::new(input), "output");
            let output = format!("{}/{}.{}", output_dir, stem, ext);
            match Self::extract_audio(input, &output, format, None) {
                Ok(path) => outputs.push(path),
                Err(e) => eprintln!("Falha ao extrair de {}: {}", input, e),
            }
        }
        Ok(outputs)
    }

    /// Returns a list of supported video file extensions for audio extraction.
    pub fn get_supported_video_extensions() -> Vec<&'static str> {
        static EXTENSIONS: LazyLock<&[&str]> = LazyLock::new(|| &["mkv", "mp4", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp", "ogv"]);
        EXTENSIONS.to_vec()
    }
}
