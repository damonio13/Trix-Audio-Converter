//! Sample rate conversion using high-quality resampling
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// Converts audio between different sample rates with optional dithering.
pub struct SampleRateConverter;

impl SampleRateConverter {
    /// Converts an audio file to the specified sample rate with optional dither.
    pub fn convert(
        input: &str,
        output: &str,
        target_rate: u32,
        dither: &str,
    ) -> Result<String, String> {
        let dither_filter = match dither {
            "triangular" => "dither=type=tri",
            "rectangular" => "dither=type=rect",
            "shaped" => "dither=type=shaped",
            _ => "",
        };

        let af = if dither_filter.is_empty() {
            format!("aresample={}", target_rate)
        } else {
            format!("aresample={},{}", target_rate, dither_filter)
        };

        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Returns a list of standard audio sample rates with descriptions.
    pub fn get_standard_rates() -> &'static [(u32, &'static str)] {
        static STATIC: LazyLock<&[(u32, &str)]> = LazyLock::new(|| {
            &[
                (8000, "8 kHz (Telephone)"),
                (11025, "11.025 kHz"),
                (16000, "16 kHz"),
                (22050, "22.05 kHz"),
                (32000, "32 kHz"),
                (44100, "44.1 kHz (CD)"),
                (48000, "48 kHz (DVD)"),
                (88200, "88.2 kHz"),
                (96000, "96 kHz (Hi-Res)"),
                (176400, "176.4 kHz"),
                (192000, "192 kHz (Hi-Res)"),
                (352800, "352.8 kHz (DXD)"),
                (384000, "384 kHz"),
            ]
        });
        &*STATIC
    }

    /// Returns supported dither algorithm names and their human-readable descriptions.
    pub fn get_dither_types() -> &'static [(&'static str, &'static str)] {
        static STATIC: LazyLock<&[(&str, &str)]> = LazyLock::new(|| {
            &[
                ("none", "Sem dithering"),
                ("triangular", "Triangular (recomendado)"),
                ("rectangular", "Rectangular"),
                ("shaped", "Shaped (melhor qualidade)"),
            ]
        });
        &*STATIC
    }
}
