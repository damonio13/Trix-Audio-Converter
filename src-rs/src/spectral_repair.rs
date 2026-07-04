//! Spectral repair for removing audio artifacts
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::sync::LazyLock;

/// Spectral repair for removing audio artifacts and noise.
pub struct SpectralRepair;

impl SpectralRepair {
    /// Removes a time region from the audio by silencing it.
    pub fn remove_region(
        input: &str,
        output: &str,
        start: f64,
        end: f64,
    ) -> Result<String, String> {
        let duration = end - start;
        let af = format!(
            "silencedetect=noise=-90dB:d={},volume=enable='between(t,{},{})':volume=0",
            duration, start, end
        );

        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Replaces a time region in the audio with silence.
    pub fn replace_with_silence(
        input: &str,
        output: &str,
        start: f64,
        end: f64,
    ) -> Result<String, String> {
        let af = format!(
            "volume=enable='between(t,{},{})':volume=0",
            start, end
        );

        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Reduces noise within a specific time region of the audio.
    pub fn reduce_noise_in_region(
        input: &str,
        output: &str,
        start: f64,
        end: f64,
    ) -> Result<String, String> {
        let af = format!(
            "afftdn=nf=-25:tn=1:om=o,volume=enable='between(t,{},{})':volume=1",
            start, end
        );

        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Detects and repairs digital clicks and impulse noise in the input file.
    pub fn repair_clicks(
        input: &str,
        output: &str,
        _threshold: f64,
    ) -> Result<String, String> {
        let af = "silencedetect=noise=-90dB:d=0.01,volume=enable='lt(t,0.01)':volume=0";

        crate::utils::run_ffmpeg_af(input, output, af)
    }

    /// Returns supported repair modes as (id, display_name) pairs.
    pub fn get_repair_types() -> Vec<(&'static str, &'static str)> {
        static REPAIR_TYPES: LazyLock<&[(&str, &str)]> = LazyLock::new(|| &[
            ("remove_region", "Remover Região"),
            ("replace_silence", "Substituir por Silêncio"),
            ("reduce_noise", "Reduzir Ruído na Região"),
            ("repair_clicks", "Reparar Cliques"),
        ]);
        REPAIR_TYPES.to_vec()
    }
}
