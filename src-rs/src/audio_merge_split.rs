//! Audio file merging, splitting, and crossfade operations.
//!
//! Combines multiple audio files into one, splits files at defined points,
//! and applies crossfade transitions between joined segments.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::path::PathBuf;
use std::sync::LazyLock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// A split point defined by a timestamp and optional label.
pub struct SplitPoint {
    pub time: f64,
    pub label: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Configuration for crossfade transitions between merged audio files.
pub struct CrossfadeConfig {
    pub duration: f64,
    pub curve: String,
}

impl Default for CrossfadeConfig {
    fn default() -> Self {
        Self {
            duration: 3.0,
            curve: "equal".into(),
        }
    }
}

/// Merges multiple audio files into a single output file.
pub struct AudioMerger;

impl AudioMerger {
    /// Merges audio files sequentially into a single output file.
    pub fn merge_files(files: &[String], output: &str) -> Result<String, String> {
        if files.is_empty() {
            return Err("No files to merge".into());
        }

        if files.len() == 1 {
            std::fs::copy(&files[0], output)
                .map_err(|e| format!("Copy failed: {}", e))?;
            return Ok(output.to_string());
        }

        let mut args = Vec::new();
        for file in files {
            args.extend(["-i", file]);
        }

        let filter = format!("concat=n={}:v=0:a=1", files.len());
        args.extend(["-filter_complex", &filter, "-y", "--", output]);

        let out = crate::utils::run_ffmpeg_raw(&args)?;

        if out.status.success() {
            Ok(output.to_string())
        } else {
            Err("Merge failed".into())
        }
    }

    /// Merges audio files with crossfade transitions between them.
    pub fn merge_with_crossfade(
        files: &[String],
        output: &str,
        config: &CrossfadeConfig,
    ) -> Result<String, String> {
        if files.len() < 2 {
            return Self::merge_files(files, output);
        }

        let mut inputs = Vec::new();
        for file in files {
            inputs.extend(["-i", file]);
        }

        let mut filter_parts = Vec::new();
        let mut last_label = "[0:a]".to_string();

        for i in 1..files.len() {
            let next_label = format!("[{}:a]", i);
            let out_label = format!("[out{}]", i);

            filter_parts.push(format!(
                "{}{}acrossfade=d={}:c1={}:c2={}{}",
                last_label, next_label, config.duration, config.curve, config.curve, out_label
            ));

            last_label = out_label;
        }

        let filter_complex = filter_parts.join(";");

        let mut args = inputs;
        args.extend(["-filter_complex", &filter_complex, "-map", &last_label, "-y", "--", output]);

        let out = crate::utils::run_ffmpeg_raw(&args)?;

        if out.status.success() {
            Ok(output.to_string())
        } else {
            Err("Crossfade merge failed".into())
        }
    }

    /// Returns the list of available crossfade curve types.
    pub fn get_crossfade_curves() -> &'static [&'static str] {
        static STATIC: LazyLock<&[&'static str]> = LazyLock::new(|| &[
            "equal", "expon", "log", "sin", "quad",
        ]);
        &*STATIC
    }
}

/// Splits audio files into segments by time or at defined split points.
pub struct AudioSplitter;

impl AudioSplitter {
    /// Splits an audio file into fixed-duration segments.
    pub fn split_by_time(input: &str, output_dir: &str, segment_duration: f64) -> Result<Vec<String>, String> {
        let output_pattern = format!("{}/segment_%03d{}", output_dir,
            PathBuf::from(input)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or(".mp3")
        );

        let seg_dur = segment_duration.to_string();

        let out = crate::utils::run_ffmpeg_raw(&[
            "-i", input,
            "-f", "segment",
            "-segment_time", &seg_dur,
            "-c", "copy",
            "-y", "--", &output_pattern,
        ])?;

        if out.status.success() {
            let mut segments = Vec::new();
            if let Ok(entries) = std::fs::read_dir(output_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("segment_") {
                        segments.push(entry.path().to_string_lossy().to_string());
                    }
                }
            }
            segments.sort();
            Ok(segments)
        } else {
            Err("Split failed".into())
        }
    }

    /// Splits an audio file at user-defined time points.
    pub fn split_at_points(input: &str, output_dir: &str, points: &[SplitPoint]) -> Result<Vec<String>, String> {
        if points.is_empty() {
            return Err("No split points defined".into());
        }

        let mut sorted_points = points.to_vec();
        sorted_points.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());

        let ext = std::path::Path::new(input)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp3");

        let mut outputs = Vec::new();
        let mut start = 0.0;

        for (i, point) in sorted_points.iter().enumerate() {
            let output_file = format!("{}/{}_{}.{}", output_dir,
                std::path::Path::new(input).file_stem().and_then(|s| s.to_str()).unwrap_or("track"),
                i + 1, ext
            );

            let duration = point.time - start;
            if duration > 0.0 {
                let out = crate::utils::run_ffmpeg_raw(&[
                    "-i", input,
                    "-ss", &start.to_string(),
                    "-t", &duration.to_string(),
                    "-c", "copy",
                    "-y", "--", &output_file,
                ])?;

                if out.status.success() {
                    outputs.push(output_file);
                }
            }

            start = point.time;
        }

        let output_file = format!("{}/{}_final.{}", output_dir,
            std::path::Path::new(input).file_stem().and_then(|s| s.to_str()).unwrap_or("track"),
            ext
        );

        let out = crate::utils::run_ffmpeg_raw(&[
            "-i", input,
            "-ss", &start.to_string(),
            "-c", "copy",
            "-y", "--", &output_file,
        ])?;

        if out.status.success() {
            outputs.push(output_file);
        }

        Ok(outputs)
    }

    /// Returns the duration of an audio file in seconds.
    pub fn get_duration(path: &str) -> Result<f64, String> {
        crate::utils::ffprobe_duration(path)
    }
}
