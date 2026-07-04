//! Loudness penalty calculator for streaming platforms.
//!
//! Estimates the gain adjustment applied by loudness normalization
//! on major streaming platforms (Spotify, YouTube, Apple Music, etc.).
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// Loudness penalty results for each major streaming platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoudnessPenalty {
    pub spotify: f64,
    pub apple_music: f64,
    pub youtube: f64,
    pub amazon: f64,
    pub tidal: f64,
    pub soundcloud: f64,
    pub peak_limiting: bool,
    pub needs_remaster: bool,
}

/// Calculates loudness penalties for various streaming platforms.
pub struct LoudnessPenaltyCalculator;

impl LoudnessPenaltyCalculator {
    /// Calculates the loudness penalty for all platforms based on input audio.
    pub fn calculate(input: &str) -> Result<LoudnessPenalty, String> {
        let result = crate::loudness::LoudnessAnalyzer::measure_lufs(input)?;

        let measured_lufs = result.input_i;
        let measured_tp = result.input_tp;

        let spotify_target = -14.0;
        let apple_target = -16.0;
        let youtube_target = -14.0;
        let amazon_target = -14.0;
        let tidal_target = -14.0;
        let soundcloud_target = -14.0;

        let spotify_penalty = (measured_lufs - spotify_target).max(0.0);
        let apple_penalty = (measured_lufs - apple_target).max(0.0);
        let youtube_penalty = (measured_lufs - youtube_target).max(0.0);
        let amazon_penalty = (measured_lufs - amazon_target).max(0.0);
        let tidal_penalty = (measured_lufs - tidal_target).max(0.0);
        let soundcloud_penalty = (measured_lufs - soundcloud_target).max(0.0);

        let peak_limiting = measured_tp > -1.0;
        let needs_remaster = spotify_penalty > 0.0 || peak_limiting;

        Ok(LoudnessPenalty {
            spotify: -spotify_penalty,
            apple_music: -apple_penalty,
            youtube: -youtube_penalty,
            amazon: -amazon_penalty,
            tidal: -tidal_penalty,
            soundcloud: -soundcloud_penalty,
            peak_limiting,
            needs_remaster,
        })
    }

    /// Returns target loudness and true peak for each supported platform.
    pub fn get_platform_targets() -> &'static [(&'static str, f64, f64)] {
        static STATIC: LazyLock<&[(&str, f64, f64)]> = LazyLock::new(|| {
            &[
                ("Spotify", -14.0, -1.0),
                ("Apple Music", -16.0, -1.0),
                ("YouTube", -14.0, -1.0),
                ("Amazon Music", -14.0, -2.0),
                ("Tidal", -14.0, -1.0),
                ("SoundCloud", -14.0, -1.0),
                ("Podcast (Apple)", -16.0, -1.0),
                ("Broadcast (EBU)", -23.0, -1.0),
            ]
        });
        &*STATIC
    }
}
