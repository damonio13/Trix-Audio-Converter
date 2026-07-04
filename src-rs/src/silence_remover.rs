//! Silence detection and removal from audio tracks
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

/// Configuration for silence detection and removal.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SilenceConfig {
    pub threshold_db: f64,
    pub min_duration: f64,
    pub keep_silence: f64,
    pub mode: String,
}

impl Default for SilenceConfig {
    fn default() -> Self {
        Self {
            threshold_db: -40.0,
            min_duration: 0.5,
            keep_silence: 0.1,
            mode: "remove".into(),
        }
    }
}

/// Represents a detected silence region in an audio file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SilenceRegion {
    pub start: f64,
    pub end: f64,
    pub duration: f64,
}

/// Detects and removes silence from audio tracks.
pub struct SilenceRemover;

impl SilenceRemover {
    /// Detects silence regions in an audio file using the given config.
    pub fn detect_silence(input: &str, config: &SilenceConfig) -> Result<Vec<SilenceRegion>, String> {
        let af = format!(
            "silencedetect=noise={}dB:d={}",
            config.threshold_db, config.min_duration
        );
        let stderr = crate::utils::run_ffmpeg_af_stderr(input, &af)?;

        let mut regions = Vec::new();
        let mut start = 0.0;

        for line in stderr.lines() {
            if line.contains("silence_start") {
                if let Some(pos) = line.find("silence_start: ") {
                    let val = line[pos + 15..].split_whitespace().next().unwrap_or("0");
                    if let Ok(t) = val.parse::<f64>() {
                        start = t;
                    }
                }
            } else if line.contains("silence_end") {
                if let Some(pos) = line.find("silence_end: ") {
                    let val = line[pos + 13..].split_whitespace().next().unwrap_or("0");
                    if let Ok(t) = val.parse::<f64>() {
                        regions.push(SilenceRegion {
                            start,
                            end: t,
                            duration: t - start,
                        });
                    }
                }
            }
        }

        Ok(regions)
    }

    /// Removes silent passages from `input` using FFmpeg's `silenceremove` filter.
    pub fn remove_silence(input: &str, output: &str, config: &SilenceConfig) -> Result<String, String> {
        let af = format!(
            "silenceremove=start_periods=1:start_duration={}:start_threshold={}dB:stop_periods=-1:stop_duration={}:stop_threshold={}dB",
            config.keep_silence, config.threshold_db, config.min_duration, config.threshold_db
        );
        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Trims leading and trailing silence from `input`.
    pub fn trim_silence(input: &str, output: &str, config: &SilenceConfig) -> Result<String, String> {
        let af = format!(
            "silenceremove=start_periods=1:start_duration=0:start_threshold={}dB:stop_periods=1:stop_duration=0:stop_threshold={}dB",
            config.threshold_db, config.threshold_db
        );
        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Pads `input` with `duration` seconds of silence at the given `position`
    /// (`"start"` inserts delay, `"end"` pads with zeros).
    pub fn add_silence(input: &str, output: &str, duration: f64, position: &str) -> Result<String, String> {
        let af = match position {
            "start" => format!("adelay=0|{}", (duration * 1000.0) as u64),
            "end" => format!("apad=pad_dur={}", duration),
            _ => format!("adelay=0|{}", (duration * 1000.0) as u64),
        };
        crate::utils::run_ffmpeg_af(input, output, &af)
    }
}
