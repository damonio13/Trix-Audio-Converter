//! Audio file analysis: bitrate, duration, codec detection, and spectrum.
//!
//! Provides comprehensive audio file metadata extraction and analysis,
//! including sample rate, channels, bit depth, and spectral information.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Complete analysis results for an audio file.
pub struct AudioAnalysis {
    pub duration: f64,
    pub sample_rate: u32,
    pub channels: u32,
    pub bit_depth: u32,
    pub codec: String,
    pub bitrate: u64,
    pub loudness_lufs: f64,
    pub true_peak: f64,
    pub dynamic_range: f64,
    pub peak_level: f64,
    pub rms_level: f64,
    pub file_size: u64,
}

/// Analyzes audio file metadata, loudness, and spectral characteristics.
pub struct AudioAnalyzer;

impl AudioAnalyzer {
    /// Performs full audio analysis including loudness, peak, and dynamic range.
    pub fn analyze(input: &str) -> Result<AudioAnalysis, String> {
        let probe_json = crate::utils::ffprobe_json(input)?;

        let probe = crate::utils::parse_audio_probe(&probe_json)?;

        let stderr = crate::utils::run_ffmpeg_af_stderr(
            input,
            "astats=metadata=1:reset=1,ametadata=print:key=lavfi.astats.Overall.RMS_level:file=-",
        )?;

        let mut rms = -99.0;
        let mut peak = -99.0;

        for line in stderr.lines() {
            if line.contains("RMS_level") {
                if let Some(val) = line.split('=').last() {
                    rms = val.trim().parse().unwrap_or(-99.0);
                }
            }
            if line.contains("Peak_level") {
                if let Some(val) = line.split('=').last() {
                    peak = val.trim().parse().unwrap_or(-99.0);
                }
            }
        }

        let lufs_result = crate::loudness::LoudnessAnalyzer::measure_lufs(input);
        let (lufs, tp) = match lufs_result {
            Ok(r) => (r.input_i, r.input_tp),
            Err(_) => (-99.0, -99.0),
        };

        let dynamic_range = if peak > -99.0 && rms > -99.0 {
            peak - rms
        } else {
            0.0
        };

        let file_size = std::fs::metadata(input).map(|m| m.len()).unwrap_or(0);

        Ok(AudioAnalysis {
            duration: probe.duration,
            sample_rate: probe.sample_rate,
            channels: probe.channels,
            bit_depth: probe.bit_depth,
            codec: probe.codec,
            bitrate: probe.bitrate,
            loudness_lufs: lufs,
            true_peak: tp,
            dynamic_range,
            peak_level: peak,
            rms_level: rms,
            file_size,
        })
    }

    /// Analyzes a batch of audio files, returning results for each.
    pub fn analyze_batch(inputs: &[String]) -> Vec<Result<AudioAnalysis, String>> {
        inputs.iter().map(|i| Self::analyze(i)).collect()
    }
}
