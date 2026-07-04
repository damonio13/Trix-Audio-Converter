//! Audio Effects & Presets
//!
//! Defines audio effects (bass boost, treble boost, reverb, pitch, etc.)
//! and provides preset configurations for common use cases.
//! Generates FFmpeg filter strings from effect parameters.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
/// Audio effect parameters for conversion processing.
pub struct AudioEffects {
    pub bass_boost: i32,
    pub treble_boost: i32,
    pub reverb: i32,
    pub chorus: bool,
    pub flanger: bool,
    pub speed: f64,
    pub pitch: i32,
    pub fade_in: f64,
    pub fade_out: f64,
    pub compressor: bool,
    pub gate: bool,
}

impl Default for AudioEffects {
    fn default() -> Self {
        Self {
            bass_boost: 0,
            treble_boost: 0,
            reverb: 0,
            chorus: false,
            flanger: false,
            speed: 1.0,
            pitch: 0,
            fade_in: 0.0,
            fade_out: 0.0,
            compressor: false,
            gate: false,
        }
    }
}

impl AudioEffects {
    /// Builds FFmpeg filter chain string from effect parameters.
    pub fn build_filters(&self) -> Vec<String> {
        let mut filters = Vec::new();

        if self.bass_boost != 0 {
            filters.push(crate::utils::build_bass_filter(self.bass_boost as f64, 100.0));
        }
        if self.treble_boost != 0 {
            filters.push(crate::utils::build_treble_filter(self.treble_boost as f64, 4000.0));
        }
        if self.reverb > 0 {
            let room = std::cmp::max(1, self.reverb / 10);
            let damp = (1.0 - self.reverb as f64 / 150.0).max(0.1);
            filters.push(format!("aecho=0.8:0.88:{}:{:.1}", room, damp));
        }
        if self.chorus {
            filters.push("chorus=0.5:0.9:50:0.4:0.25:2".into());
        }
        if self.flanger {
            filters.push("flanger".into());
        }
        if (self.speed - 1.0).abs() > f64::EPSILON {
            filters.push(crate::utils::build_tempo_filter(self.speed));
        }
        if self.pitch != 0 {
            let cents = self.pitch * 100;
            let ratio = 1.0 + cents as f64 / 1200.0;
            filters.push(format!("rubberband=pitch={:.4}", ratio));
        }
        if self.fade_in > 0.0 {
            filters.push(format!("afade=t=in:st=0:d={}", self.fade_in));
        }
        if self.fade_out > 0.0 {
            filters.push(format!("afade=t=out:st=0:d={}", self.fade_out));
        }
        if self.compressor {
            filters.push("acompressor=threshold=-20dB:ratio=4:attack=5:release=50".into());
        }
        if self.gate {
            filters.push("agate=threshold=-40dB:ratio=2:attack=10:release=50".into());
        }

        filters
    }

    /// Returns true if any audio effects are enabled (non-default values).
    pub fn has_effects(&self) -> bool {
        self.bass_boost != 0
            || self.treble_boost != 0
            || self.reverb > 0
            || self.chorus
            || self.flanger
            || (self.speed - 1.0).abs() > f64::EPSILON
            || self.pitch != 0
            || self.fade_in > 0.0
            || self.fade_out > 0.0
            || self.compressor
            || self.gate
    }
}

/// Returns a map of named effect presets (bass_boost, vocal_enhance, nightcore, etc.).
pub fn get_effect_presets() -> &'static HashMap<String, HashMap<String, serde_json::Value>> {
    static PRESETS: OnceLock<HashMap<String, HashMap<String, serde_json::Value>>> = OnceLock::new();
    PRESETS.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("none".into(), HashMap::new());
        m.insert("bass_boost".into(), HashMap::from([("bass_boost".into(), serde_json::json!(8))]));
        m.insert("treble_boost".into(), HashMap::from([("treble_boost".into(), serde_json::json!(6))]));
        m.insert("vocal_enhance".into(), HashMap::from([
            ("bass_boost".into(), serde_json::json!(3)),
            ("treble_boost".into(), serde_json::json!(4)),
            ("compressor".into(), serde_json::json!(true)),
        ]));
        m.insert("warm".into(), HashMap::from([
            ("bass_boost".into(), serde_json::json!(5)),
            ("treble_boost".into(), serde_json::json!(-2)),
            ("reverb".into(), serde_json::json!(15)),
        ]));
        m.insert("bright".into(), HashMap::from([
            ("treble_boost".into(), serde_json::json!(8)),
            ("compressor".into(), serde_json::json!(true)),
        ]));
        m.insert("live".into(), HashMap::from([
            ("reverb".into(), serde_json::json!(30)),
            ("chorus".into(), serde_json::json!(true)),
        ]));
        m.insert("vintage".into(), HashMap::from([
            ("bass_boost".into(), serde_json::json!(4)),
            ("treble_boost".into(), serde_json::json!(-3)),
            ("reverb".into(), serde_json::json!(20)),
            ("compressor".into(), serde_json::json!(true)),
        ]));
        m.insert("podcast".into(), HashMap::from([
            ("bass_boost".into(), serde_json::json!(3)),
            ("treble_boost".into(), serde_json::json!(5)),
            ("compressor".into(), serde_json::json!(true)),
            ("gate".into(), serde_json::json!(true)),
        ]));
        m.insert("nightcore".into(), HashMap::from([
            ("speed".into(), serde_json::json!(1.25)),
            ("pitch".into(), serde_json::json!(3)),
        ]));
        m.insert("slowed".into(), HashMap::from([
            ("speed".into(), serde_json::json!(0.8)),
            ("pitch".into(), serde_json::json!(-2)),
            ("reverb".into(), serde_json::json!(25)),
        ]));
        m
    })
}
