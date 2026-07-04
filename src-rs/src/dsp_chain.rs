//! Digital signal processing chain: chains multiple audio effects.
//!
//! Manages an ordered pipeline of DSP processing steps (eq, compressor,
//! limiter, etc.) that are applied sequentially to audio buffers.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// A single DSP effect with type, parameters, and enabled state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspEffect {
    pub effect_type: String,
    pub params: HashMap<String, f64>,
    pub enabled: bool,
}

/// An ordered pipeline of DSP effects applied sequentially to audio.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DspChain {
    pub effects: Vec<DspEffect>,
}

impl DspChain {
    /// Adds a new effect to the chain with the given type and parameters.
    pub fn add_effect(&mut self, effect_type: &str, params: HashMap<String, f64>) {
        self.effects.push(DspEffect {
            effect_type: effect_type.to_string(),
            params,
            enabled: true,
        });
    }

    /// Builds the ffmpeg filter chain string from all enabled effects.
    pub fn build_filter_chain(&self) -> String {
        let mut filters = Vec::new();

        for effect in &self.effects {
            if !effect.enabled {
                continue;
            }

            let filter = match crate::utils::build_effect_filter(&effect.effect_type, &effect.params) {
                Some(f) => f,
                None => continue,
            };

            filters.push(filter);
        }

        filters.join(",")
    }

    /// Applies the entire effect chain to an audio file.
    pub fn apply_chain(
        &self,
        input: &str,
        output: &str,
    ) -> Result<String, String> {
        let af = self.build_filter_chain();
        if af.is_empty() {
            return Err("Nenhum efeito habilitado".into());
        }

        crate::utils::run_ffmpeg_af(input, output, &af)
    }

    /// Returns all available DSP effects with their parameter definitions.
    pub fn get_available_effects() -> Vec<(&'static str, &'static str, Vec<(&'static str, f64, f64, f64)>)> {
        static STATIC: LazyLock<Vec<(&str, &str, Vec<(&str, f64, f64, f64)>)>> = LazyLock::new(|| {
            vec![
                ("equalizer", "Equalizador", vec![
                    ("frequency", 1000.0, 20.0, 20000.0),
                    ("gain", 0.0, -20.0, 20.0),
                    ("q", 1.0, 0.1, 10.0),
                ]),
                ("bass", "Graves", vec![
                    ("gain", 0.0, -20.0, 20.0),
                    ("frequency", 100.0, 20.0, 500.0),
                ]),
                ("treble", "Agudos", vec![
                    ("gain", 0.0, -20.0, 20.0),
                    ("frequency", 3000.0, 1000.0, 20000.0),
                ]),
                ("reverb", "Reverb", vec![
                    ("roomsize", 0.5, 0.0, 1.0),
                    ("wet", 0.3, 0.0, 1.0),
                ]),
                ("chorus", "Chorus", vec![
                    ("delay", 50.0, 0.0, 100.0),
                    ("depth", 0.5, 0.0, 1.0),
                    ("speed", 0.5, 0.1, 2.0),
                ]),
                ("flanger", "Flanger", vec![
                    ("delay", 0.5, 0.0, 1.0),
                    ("depth", 0.5, 0.0, 1.0),
                ]),
                ("phaser", "Phaser", vec![
                    ("speed", 0.5, 0.1, 2.0),
                    ("depth", 0.5, 0.0, 1.0),
                ]),
                ("compand", "Compressor", vec![
                    ("threshold", -20.0, -60.0, 0.0),
                    ("ratio", 4.0, 1.0, 20.0),
                ]),
                ("limiter", "Limiter", vec![
                    ("limit", -1.0, -10.0, 0.0),
                ]),
                ("highpass", "Filtro Passa-Alta", vec![
                    ("frequency", 200.0, 20.0, 10000.0),
                ]),
                ("lowpass", "Filtro Passa-Baixa", vec![
                    ("frequency", 3000.0, 100.0, 20000.0),
                ]),
                ("volume", "Volume", vec![
                    ("gain", 1.0, 0.0, 5.0),
                ]),
                ("tempo", "Tempo", vec![
                    ("speed", 1.0, 0.5, 2.0),
                ]),
                ("pitch", "Pitch", vec![
                    ("semitones", 0.0, -12.0, 12.0),
                ]),
            ]
        });
        STATIC.clone()
    }
}
