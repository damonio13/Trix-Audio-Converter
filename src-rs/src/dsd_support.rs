//! DSD (DSD64/DSD128/DSD256) audio format support.
//!
//! Handles reading and decoding Direct Stream Digital audio formats
//! at various rates for high-resolution audio conversion.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use std::sync::LazyLock;

/// DSD (Direct Stream Digital) format support.
pub struct DsdSupport;

impl DsdSupport {
    /// Converts DSD audio to PCM at the specified bit depth.
    pub fn dsd_to_pcm(input: &str, output: &str, bit_depth: u32) -> Result<String, String> {
        let codec = match bit_depth {
            16 => "pcm_s16le",
            24 => "pcm_s24le",
            _ => "pcm_s32le",
        };

        crate::utils::run_ffmpeg_raw_checked(&[
            "-hide_banner",
            "-i", input,
            "-acodec", codec,
            "-ar", "44100",
            "-y", "--", output,
        ])?;

        Ok("Sucesso".into())
    }

    /// Converts PCM audio to DSD at the specified sample rate.
    pub fn pcm_to_dsd(input: &str, output: &str, dsd_rate: &str) -> Result<String, String> {
        let rate_val: u32 = dsd_rate.parse().unwrap_or(2822400);
        crate::utils::run_ffmpeg_raw_checked(&[
            "-hide_banner",
            "-i", input,
            "-acodec", "dsd_lsbf_planar",
            "-ar", &rate_val.to_string(),
            "-y", "--", output,
        ])?;

        Ok("Sucesso".into())
    }

    /// Returns supported DSD formats with sample rates.
    pub fn get_dsd_formats() -> Vec<(&'static str, &'static str, u32)> {
        static STATIC: LazyLock<&[(&str, &str, u32)]> = LazyLock::new(|| {
            &[
                ("dsf", "DSD64 (2.8MHz)", 2822400),
                ("dff", "DSD64 (2.8MHz)", 2822400),
                ("dsf", "DSD128 (5.6MHz)", 5644800),
                ("dsf", "DSD256 (11.2MHz)", 11289600),
                ("dsf", "DSD512 (22.4MHz)", 22579200),
            ]
        });
        STATIC.to_vec()
    }

    /// Returns available bit depths for PCM conversion.
    pub fn get_bit_depths() -> Vec<(u32, &'static str)> {
        static STATIC: LazyLock<&[(u32, &str)]> = LazyLock::new(|| {
            &[
                (16, "16-bit (CD)"),
                (24, "24-bit (Hi-Res)"),
                (32, "32-bit (Studio)"),
            ]
        });
        STATIC.to_vec()
    }
}
