//! Dynamic range measurement and classification.
//!
//! Computes the dynamic range of audio tracks and classifies them
//! using the ITU-R BS1770 standard for broadcast compliance.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::LazyLock;

/// Result of dynamic range measurement with classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRangeResult {
    pub dr_value: f64,
    pub peak: f64,
    pub rms: f64,
    pub crest_factor: f64,
    pub dynamic_range_db: f64,
    pub classification: Cow<'static, str>,
}

/// Measures and classifies the dynamic range of audio tracks.
pub struct DynamicRangeMeter;

impl DynamicRangeMeter {
    /// Measures dynamic range and returns detailed results with classification.
    pub fn measure(input: &str) -> Result<DynamicRangeResult, String> {
        let (peak, rms) = crate::utils::measure_peak_rms_db(input)?;
        let crest_factor = peak - rms;

        let dr_value = (crest_factor / 2.0).round();
        let dynamic_range_db = crest_factor;

        let classification: Cow<'static, str> = match (dr_value.max(0.0) as u32).min(20) {
            0..=3 => "Muito Comprimido (Metal/Pop)".into(),
            4..=6 => "Comprimido (Rock/Pop)".into(),
            7..=10 => "Moderado (Jazz/Classical)".into(),
            11..=14 => "Dinâmico (Classical/Live)".into(),
            15..=20 => "Muito Dinâmico (Audiophile)".into(),
            _ => "Extremamente Dinâmico".into(),
        };

        Ok(DynamicRangeResult {
            dr_value,
            peak,
            rms,
            crest_factor,
            dynamic_range_db,
            classification,
        })
    }

    /// Returns dynamic range classification ranges and labels.
    pub fn get_classifications() -> Vec<(f64, f64, &'static str)> {
        static STATIC: LazyLock<&[(f64, f64, &str)]> = LazyLock::new(|| {
            &[
                (0.0, 3.0, "Muito Comprimido (Metal/Pop)"),
                (4.0, 6.0, "Comprimido (Rock/Pop)"),
                (7.0, 10.0, "Moderado (Jazz/Classical)"),
                (11.0, 14.0, "Dinâmico (Classical/Live)"),
                (15.0, 20.0, "Muito Dinâmico (Audiophile)"),
            ]
        });
        STATIC.to_vec()
    }
}
