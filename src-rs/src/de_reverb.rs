//! De-reverb effect for room echo reduction.
//!
//! Analyzes and suppresses room reverberation artifacts from audio
//! recordings to improve clarity and intelligibility.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use std::sync::LazyLock;

/// De-reverb effect for room echo reduction.
pub struct DeReverb;

impl DeReverb {
    /// Applies de-reverb with strength and frequency parameters.
    pub fn apply(
        input: &str,
        output: &str,
        _strength: f64,
        frequency: f64,
    ) -> Result<String, String> {
        let af = format!(
            "highpass=f={}:poles=2,arnndn=model=cb.rnnn,agate=threshold=-40dB:ratio=2:attack=5:release=50",
            frequency
        );

        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Applies advanced de-reverb with room size, damping, and wet level.
    pub fn apply_advanced(
        input: &str,
        output: &str,
        room_size: f64,
        damping: f64,
        wet: f64,
    ) -> Result<String, String> {
        let af = format!(
            "freeverb=roomsize={}:damp={}:wet={}:dry=1:width=0.5:freeze=0",
            room_size, damping, wet
        );

        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Returns available de-reverb presets with descriptions and strength values.
    pub fn get_presets() -> Vec<(&'static str, &'static str, f64)> {
        static STATIC: LazyLock<&[(&str, &str, f64)]> = LazyLock::new(|| {
            &[
                ("subtle", "Sutil (sala pequena)", 0.3),
                ("medium", "Médio (sala média)", 0.5),
                ("heavy", "Forte (sala grande)", 0.8),
                ("bathroom", "Banheiro (muito reflexo)", 0.9),
                ("studio", "Estúdio (seco)", 0.2),
            ]
        });
        STATIC.to_vec()
    }
}
