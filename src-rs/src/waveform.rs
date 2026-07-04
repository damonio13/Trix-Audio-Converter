//! Waveform image generation for audio visualization
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// Generates waveform images for audio visualization.
pub struct WaveformGenerator;

fn is_valid_color(s: &str) -> bool {
    s.len() <= 20 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '#' || c == '_' || c == '-')
}

impl WaveformGenerator {
    /// Generates a waveform image (PNG) from an audio file.
    pub fn generate_image(
        input: &str,
        output: &str,
        width: u32,
        height: u32,
        color: &str,
        bg_color: &str,
    ) -> Result<String, String> {
        let safe_color = if is_valid_color(color) { color } else { "#ffffff" };
        let safe_bg = if is_valid_color(bg_color) { bg_color } else { "#000000" };
        let filter = format!(
            "showwavespic=s={}x{}:colors={}:split_channels=0:split_channels_color={}",
            width, height, safe_color, safe_bg
        );

        crate::utils::run_ffmpeg_raw_checked(&[
            "-hide_banner",
            "-i", input,
            "-filter_complex", &filter,
            "-frames:v", "1",
            "-y", "--", output,
        ])?;

        Ok("Sucesso".into())
    }

    /// Generates a waveform image in SVG format from an audio file.
    pub fn generate_svg(
        input: &str,
        output: &str,
        width: u32,
        height: u32,
        color: &str,
    ) -> Result<String, String> {
        let safe_color = if is_valid_color(color) { color } else { "#ffffff" };
        let filter = format!(
            "showwavespic=s={}x{}:colors={}:split_channels=0",
            width, height, safe_color
        );

        crate::utils::run_ffmpeg_raw_checked(&[
            "-hide_banner",
            "-i", input,
            "-filter_complex", &filter,
            "-frames:v", "1",
            "-y", "--", output,
        ])?;

        Ok("Sucesso".into())
    }

    /// Generates raw amplitude data samples from the input audio file.
    pub fn generate_data(input: &str, samples: u32) -> Result<Vec<f64>, String> {
        let af = format!("showwavespic=s={}x1:colors=white", samples);
        let out = crate::utils::run_ffmpeg_raw(&[
            "-hide_banner",
            "-i", input,
            "-af", &af,
            "-frames:v", "1",
            "-f", "rawvideo",
            "-",
        ])?;

        let data: Vec<f64> = out.stdout
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    let val = u16::from_be_bytes([chunk[0], chunk[1]]);
                    (val as f64 / u16::MAX as f64) * 2.0 - 1.0
                } else {
                    0.0
                }
            })
            .collect();

        Ok(data)
    }

    /// Returns available waveform color presets as (name, foreground, background) tuples.
    pub fn get_color_presets() -> Vec<(&'static str, &'static str, &'static str)> {
        static COLOR_PRESETS: LazyLock<&[(&str, &str, &str)]> = LazyLock::new(|| &[
            ("cyan", "#00FFFF", "#000000"),
            ("green", "#00FF00", "#000000"),
            ("magenta", "#FF00FF", "#000000"),
            ("yellow", "#FFFF00", "#000000"),
            ("white", "#FFFFFF", "#000000"),
            ("red", "#FF0000", "#000000"),
            ("blue", "#0000FF", "#FFFFFF"),
            ("orange", "#FF8800", "#000000"),
        ]);
        COLOR_PRESETS.to_vec()
    }
}
