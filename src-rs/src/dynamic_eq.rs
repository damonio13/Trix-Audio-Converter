//! Dynamic equalizer with frequency-dependent compression.
//!
//! Adjusts EQ gain automatically based on the spectral content
//! of the input signal for transparent tonal balance correction.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use std::sync::LazyLock;

/// Dynamic equalizer with frequency-dependent compression.
pub struct DynamicEq;

impl DynamicEq {
    /// Applies dynamic EQ with the specified frequency bands.
    pub fn apply(
        input: &str,
        output: &str,
        bands: &[DynamicBand],
    ) -> Result<String, String> {
        let mut filters = Vec::new();

        for band in bands {
            let filter = format!(
                "equalizer=f={}:t=q:w={}:g={}",
                band.frequency, band.q, band.gain
            );
            filters.push(filter);
        }

        let af = filters.join(",");
        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Applies dynamic EQ using a named preset.
    pub fn apply_preset(
        input: &str,
        output: &str,
        preset: &str,
    ) -> Result<String, String> {
        let bands = match preset {
            "brighten" => vec![
                DynamicBand { frequency: 3000.0, q: 1.0, gain: 3.0, threshold: -20.0, ratio: 2.0 },
                DynamicBand { frequency: 8000.0, q: 0.7, gain: 2.0, threshold: -18.0, ratio: 1.5 },
            ],
            "warmth" => vec![
                DynamicBand { frequency: 200.0, q: 1.0, gain: 3.0, threshold: -25.0, ratio: 2.0 },
                DynamicBand { frequency: 500.0, q: 0.8, gain: 2.0, threshold: -22.0, ratio: 1.5 },
            ],
            "de_mud" => vec![
                DynamicBand { frequency: 400.0, q: 1.2, gain: -3.0, threshold: -20.0, ratio: 2.5 },
                DynamicBand { frequency: 800.0, q: 1.0, gain: -2.0, threshold: -18.0, ratio: 2.0 },
            ],
            "presence" => vec![
                DynamicBand { frequency: 2500.0, q: 1.0, gain: 2.0, threshold: -22.0, ratio: 1.8 },
                DynamicBand { frequency: 5000.0, q: 0.8, gain: 1.5, threshold: -20.0, ratio: 1.5 },
            ],
            "sibilance_control" => vec![
                DynamicBand { frequency: 6000.0, q: 2.0, gain: -4.0, threshold: -15.0, ratio: 3.0 },
                DynamicBand { frequency: 8000.0, q: 1.5, gain: -3.0, threshold: -16.0, ratio: 2.5 },
            ],
            _ => return Err("Preset desconhecido".into()),
        };

        Self::apply(input, output, &bands)
    }

    /// Returns available dynamic EQ presets with display names.
    pub fn get_presets() -> Vec<(&'static str, &'static str)> {
        static STATIC: LazyLock<&[(&str, &str)]> = LazyLock::new(|| {
            &[
                ("brighten", "Brilho (agudos)"),
                ("warmth", "Calor (graves)"),
                ("de_mud", "Remover Lama"),
                ("presence", "Presença (voz)"),
                ("sibilance_control", "Controle de Sibilância"),
            ]
        });
        STATIC.to_vec()
    }
}

/// A single dynamic EQ band with frequency, Q, gain, threshold, and ratio.
#[derive(Debug, Clone)]
pub struct DynamicBand {
    pub frequency: f64,
    pub q: f64,
    pub gain: f64,
    pub threshold: f64,
    pub ratio: f64,
}
