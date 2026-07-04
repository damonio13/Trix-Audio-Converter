//! Audio Preview & Waveform
//!
//! Generates audio previews and waveform visualizations using FFmpeg.
//! Returns PCM data for frontend waveform rendering.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};

/// Result of generating an audio preview clip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResult {
    pub success: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}

/// Waveform data result with PCM samples and duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformResult {
    pub success: bool,
    pub waveform: Vec<f64>,
    pub duration: f64,
    pub error: Option<String>,
}

/// Generates audio previews and waveform data via FFmpeg.
pub struct AudioPreview;

impl AudioPreview {
    /// Generates a short audio preview clip using FFmpeg.
    pub fn generate_preview(input_path: &str, start_sec: f64, duration_sec: f64) -> PreviewResult {
        if !crate::utils::is_safe_path(input_path) {
            return PreviewResult { success: false, path: None, error: Some("Invalid input path".into()) };
        }

        let temp_dir = std::env::current_dir().unwrap_or_default().join("temp");
        let _ = std::fs::create_dir_all(&temp_dir);

        let output_name = format!("preview_{}.mp3", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
        let output_path = temp_dir.join(&output_name);

        let result = crate::utils::run_ffmpeg_raw(&[
            "-y", "-hide_banner", "-loglevel", "error",
            "-ss", &start_sec.to_string(),
            "-t", &duration_sec.to_string(),
            "-i", input_path,
            "-vn",
            "-codec:a", "libmp3lame",
            "-b:a", "128k",
            "-ar", "44100",
            "-ac", "2",
            "--",
            &output_path.to_string_lossy(),
        ]);

        match result {
            Ok(output) if output.status.success() && output_path.exists() => {
                PreviewResult {
                    success: true,
                    path: Some(output_path.to_string_lossy().to_string()),
                    error: None,
                }
            }
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                PreviewResult {
                    success: false,
                    path: None,
                    error: Some(if err.len() > 200 { err[..200].to_string() } else { err.to_string() }),
                }
            }
            Err(e) => PreviewResult {
                success: false,
                path: None,
                error: Some(e.to_string()),
            },
        }
    }

    /// Deletes temporary preview files from the system.
    pub fn cleanup_preview(preview_path: &str) {
        if !crate::utils::is_safe_path(preview_path) {
            return;
        }
        if let Ok(canonical) = std::fs::canonicalize(preview_path) {
            let temp_dir = std::env::current_dir().unwrap_or_default().join("temp");
            if let Ok(temp_canonical) = std::fs::canonicalize(&temp_dir) {
                if canonical.starts_with(&temp_canonical) {
                    let _ = std::fs::remove_file(&canonical);
                }
            }
        }
    }

    /// Extracts PCM waveform data for visualization.
    pub fn get_waveform_data(file_path: &str, samples: usize) -> WaveformResult {
        let duration = Self::get_duration(file_path);
        if duration <= 0.0 {
            return WaveformResult {
                success: true,
                waveform: vec![0.0; samples],
                duration: 0.0,
                error: None,
            };
        }

        let result = crate::utils::run_ffmpeg_raw(&[
            "-hide_banner", "-nostats",
            "-i", file_path,
            "-filter:a", &format!("aresample=8000,asetnsamples=n={}", samples),
            "-f", "u8", "-ac", "1", "-y",
            "--", "pipe:1",
        ]);

        match result {
            Ok(output) if output.status.success() && !output.stdout.is_empty() => {
                let waveform: Vec<f64> = output.stdout.iter()
                    .take(samples)
                    .map(|&b| (b as f64 / 128.0).min(1.0))
                    .collect();

                let mut wf = waveform;
                while wf.len() < samples {
                    wf.push(0.0);
                }

                WaveformResult {
                    success: true,
                    waveform: wf,
                    duration,
                    error: None,
                }
            }
            _ => WaveformResult {
                success: true,
                waveform: vec![0.0; samples],
                duration,
                error: None,
            },
        }
    }

    /// Returns the duration of an audio file in seconds.
    pub fn get_duration(file_path: &str) -> f64 {
        if !crate::utils::is_safe_path(file_path) {
            return 0.0;
        }
        crate::utils::ffprobe_duration(file_path).unwrap_or(0.0)
    }
}
