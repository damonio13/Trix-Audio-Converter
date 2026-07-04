//! Loudness measurement and normalization (LUFS).
//!
//! Implements ITU-R BS1770 / EBU R128 loudness measurement and
//! provides normalization to target integrated loudness levels.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub use crate::utils::LoudnormResult as LoudnessResult;

/// ReplayGain analysis result with track and album gain values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayGainResult {
    pub track_gain: f64,
    pub track_peak: f64,
    pub album_gain: f64,
    pub album_peak: f64,
}

/// Measures and normalizes audio loudness using LUFS standards.
pub struct LoudnessAnalyzer;

impl LoudnessAnalyzer {
    /// Measures integrated loudness (LUFS) of the input audio file.
    pub fn measure_lufs(input: &str) -> Result<LoudnessResult, String> {
        let stderr = crate::utils::run_ffmpeg_af_stderr(
            input,
            "loudnorm=I=-16:TP=-1.5:LRA=11:print_format=json",
        )?;

        crate::utils::parse_loudnorm_json(&stderr)
    }

    /// Normalizes audio to target LUFS, true peak, and loudness range.
    pub fn normalize_to_lufs(
        input: &str,
        output: &str,
        target_lufs: f64,
        target_tp: f64,
        target_lra: f64,
    ) -> Result<String, String> {
        let measured = Self::measure_lufs(input)?;

        let af = format!(
            "loudnorm=I={}:TP={}:LRA={}:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset={}:linear=true",
            target_lufs, target_tp, target_lra,
            measured.input_i, measured.input_tp, measured.input_lra,
            measured.input_thresh, measured.target_offset
        );
        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Returns loudness normalization presets for various use cases.
    pub fn get_presets() -> &'static [(&'static str, f64, f64, f64)] {
        static STATIC: LazyLock<&[(&str, f64, f64, f64)]> = LazyLock::new(|| {
            &[
                ("Streaming (Spotify/YouTube)", -14.0, -1.0, 11.0),
                ("Broadcast (EBU R128)", -23.0, -1.0, 7.0),
                ("Podcast / Audiobook", -16.0, -1.5, 11.0),
                ("Música (Alta Qualidade)", -14.0, -0.8, 8.0),
                ("Música (Dinâmica)", -18.0, -2.0, 12.0),
                ("Cinema / Filme", -24.0, -2.0, 7.0),
                ("Jogo", -16.0, -1.0, 10.0),
                ("Anúncio / Commercial", -10.0, -0.5, 5.0),
            ]
        });
        &*STATIC
    }
}

/// Calculates and applies ReplayGain for volume normalization.
pub struct ReplayGain;

impl ReplayGain {
    /// Calculates ReplayGain values for the input audio file.
    pub fn calculate(input: &str) -> Result<ReplayGainResult, String> {
        let stderr = crate::utils::run_ffmpeg_af_stderr(input, "replaygain")?;

        let mut track_gain = 0.0;
        let mut track_peak = 0.0;

        for line in stderr.lines() {
            if line.contains("track_gain") {
                if let Some(val) = line.split('=').last() {
                    track_gain = val.trim().replace(" dB", "").parse().unwrap_or(0.0);
                }
            }
            if line.contains("track_peak") {
                if let Some(val) = line.split('=').last() {
                    track_peak = val.trim().parse().unwrap_or(0.0);
                }
            }
        }

        Ok(ReplayGainResult {
            track_gain,
            track_peak,
            album_gain: track_gain,
            album_peak: track_peak,
        })
    }

    /// Applies a gain adjustment in dB to the audio file.
    pub fn apply_gain(input: &str, output: &str, gain_db: f64) -> Result<String, String> {
        let volume = 10.0_f64.powf(gain_db / 20.0);
        let af = format!("volume={:.6}", volume);
        crate::utils::run_ffmpeg_af(input, output, &af)
    }
}
