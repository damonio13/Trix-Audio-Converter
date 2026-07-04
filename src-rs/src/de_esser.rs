//! De-esser effect for sibilance reduction.
//!
//! Attenuates harsh sibilant consonants (s, sh, ch sounds) in vocal
//! recordings using frequency-selective dynamic compression.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// De-esser for sibilance reduction in vocal recordings.
pub struct DeEsser;

impl DeEsser {
    /// Applies de-esser with custom frequency, threshold, and ratio.
    pub fn apply(
        input: &str,
        output: &str,
        frequency: f64,
        threshold: f64,
        ratio: f64,
    ) -> Result<String, String> {
        let af = format!(
            "bandpass=f={}:width_type=q:w=2,acompressor=threshold={}dB:ratio={}:attack=0.1:release=50,volume=1.5",
            frequency, threshold, ratio
        );

        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Applies de-esser using a named preset (gentle, medium, aggressive, etc.).
    pub fn apply_preset(
        input: &str,
        output: &str,
        preset: &str,
    ) -> Result<String, String> {
        let (freq, threshold, ratio) = match preset {
            "gentle" => (6000.0, -20.0, 2.0),
            "medium" => (5500.0, -18.0, 3.0),
            "aggressive" => (5000.0, -15.0, 4.0),
            "female_vocal" => (7000.0, -22.0, 2.5),
            "male_vocal" => (5000.0, -18.0, 3.0),
            _ => return Err("Preset desconhecido".into()),
        };

        Self::apply(input, output, freq, threshold, ratio)
    }

    /// Returns available de-esser presets with display names.
    pub fn get_presets() -> &'static [(&'static str, &'static str)] {
        static STATIC: LazyLock<&[(&'static str, &'static str)]> = LazyLock::new(|| &[
            ("gentle", "Suave"),
            ("medium", "Médio"),
            ("aggressive", "Agressivo"),
            ("female_vocal", "Voz Feminina"),
            ("male_vocal", "Voz Masculina"),
        ]);
        &*STATIC
    }
}
