//! Spectrogram image generation from audio files
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

/// Configuration for spectrogram image generation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpectrogramConfig {
    pub width: u32,
    pub height: u32,
    pub color: String,
    pub scale: String,
    pub min_freq: f64,
    pub max_freq: f64,
}

impl Default for SpectrogramConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 400,
            color: "magma".into(),
            scale: "log".into(),
            min_freq: 20.0,
            max_freq: 20000.0,
        }
    }
}

use std::sync::LazyLock;

/// Generates spectrogram images from audio files.
pub struct Spectrogram;

impl Spectrogram {
    /// Generates a spectrogram image from an audio file.
    pub fn generate(input: &str, output: &str, config: &SpectrogramConfig) -> Result<String, String> {
        let color_map = match config.color.as_str() {
            "magma" => "1",
            "inferno" => "2",
            "plasma" => "3",
            "viridis" => "4",
            "cividis" => "5",
            "turbo" => "6",
            _ => "1",
        };

        let safe_scale = match config.scale.as_str() {
            "log" | "lin" | "sqrt" | "cbrt" => config.scale.as_str(),
            _ => "log",
        };

        let lavfi = format!(
            "showspectrumpic=s={}x{}:mode=combined:color={}:fscale={}:win_size=2046",
            config.width, config.height, color_map, safe_scale
        );

        crate::utils::run_ffmpeg_raw_checked(&[
            "-i", input,
            "-lavfi", &lavfi,
            "-y", "--", output,
        ])?;

        Ok(output.to_string())
    }

    /// Generates an animated spectrogram video from input using FFmpeg's showspectrumpic filter.
    pub fn generate_live(input: &str, output: &str) -> Result<String, String> {
        crate::utils::run_ffmpeg_raw_checked(&[
            "-i", input,
            "-lavfi", "showspectrum=s=800x400:mode=combined:color=intensity:fscale=lin",
            "-y", "--", output,
        ])?;

        Ok(output.to_string())
    }

    /// Returns the list of available colour schemes for spectrogram rendering.
    pub fn get_available_colors() -> Vec<&'static str> {
        static COLORS: LazyLock<&[&str]> = LazyLock::new(|| &["magma", "inferno", "plasma", "viridis", "cividis", "turbo"]);
        COLORS.to_vec()
    }
}
