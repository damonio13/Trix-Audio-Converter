//! Parametric equalizer with presets and custom bands.
//!
//! Provides configurable biquad filter bands with built-in presets
//! for common mastering and restoration tasks.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use std::sync::LazyLock;

/// A single parametric EQ band with frequency, gain, and Q factor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EqualizerBand {
    pub frequency: f64,
    pub gain: f64,
    pub q: f64,
}

/// Parametric equalizer with configurable bands and presets.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Equalizer {
    pub enabled: bool,
    pub bands: Vec<EqualizerBand>,
    pub preset: Option<String>,
}

impl Default for Equalizer {
    fn default() -> Self {
        Self {
            enabled: false,
            bands: vec![
                EqualizerBand { frequency: 31.0, gain: 0.0, q: 0.7 },
                EqualizerBand { frequency: 62.0, gain: 0.0, q: 0.7 },
                EqualizerBand { frequency: 125.0, gain: 0.0, q: 0.7 },
                EqualizerBand { frequency: 250.0, gain: 0.0, q: 0.7 },
                EqualizerBand { frequency: 500.0, gain: 0.0, q: 0.7 },
                EqualizerBand { frequency: 1000.0, gain: 0.0, q: 0.7 },
                EqualizerBand { frequency: 2000.0, gain: 0.0, q: 0.7 },
                EqualizerBand { frequency: 4000.0, gain: 0.0, q: 0.7 },
                EqualizerBand { frequency: 8000.0, gain: 0.0, q: 0.7 },
                EqualizerBand { frequency: 16000.0, gain: 0.0, q: 0.7 },
            ],
            preset: None,
        }
    }
}

impl Equalizer {
    /// Returns all available EQ presets with gain values for 10 bands.
    pub fn get_presets() -> &'static Vec<(&'static str, Vec<f64>)> {
        static STATIC: LazyLock<Vec<(&str, Vec<f64>)>> = LazyLock::new(|| {
            vec![
                ("Flat", vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
                ("Bass Boost", vec![6.0, 5.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
                ("Treble Boost", vec![0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 4.0, 5.0, 6.0, 6.0]),
                ("Vocal", vec![-2.0, -1.0, 0.0, 2.0, 4.0, 4.0, 3.0, 2.0, 1.0, 0.0]),
                ("Rock", vec![5.0, 4.0, 2.0, 0.0, -1.0, -1.0, 0.0, 2.0, 3.0, 4.0]),
                ("Pop", vec![-1.0, 2.0, 4.0, 4.0, 2.0, 0.0, -1.0, -1.0, 2.0, 3.0]),
                ("Jazz", vec![3.0, 2.0, 0.0, 2.0, -2.0, -2.0, 0.0, 2.0, 3.0, 4.0]),
                ("Electronic", vec![5.0, 4.0, 1.0, 0.0, -2.0, 0.0, 1.0, 3.0, 4.0, 5.0]),
                ("Hip Hop", vec![5.0, 4.0, 2.0, 0.0, -1.0, -1.0, 1.0, 0.0, 2.0, 3.0]),
                ("Podcast", vec![-2.0, 0.0, 2.0, 4.0, 4.0, 3.0, 2.0, 1.0, 0.0, -1.0]),
                ("Loudness", vec![6.0, 4.0, 0.0, -2.0, -1.0, 0.0, -1.0, -2.0, 3.0, 5.0]),
                ("SBAcoustic", vec![4.0, 3.0, 1.0, 2.0, 3.0, 3.0, 2.0, 1.0, 2.0, 3.0]),
                ("Headphones", vec![4.0, 7.0, 4.0, -1.0, -2.0, 1.0, 3.0, 5.0, 6.0, 6.0]),
                ("Bass Cut", vec![-6.0, -4.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
                ("Laptop Speakers", vec![4.0, 7.0, 5.0, 2.0, -1.0, -1.0, 1.0, 3.0, 5.0, 6.0]),
            ]
        });
        &STATIC
    }

    /// Applies a named preset to this equalizer instance.
    pub fn apply_preset(&mut self, preset_name: &str) {
        for (name, gains) in Self::get_presets() {
            if *name == preset_name {
                for (i, gain) in gains.iter().enumerate() {
                    if i < self.bands.len() {
                        self.bands[i].gain = *gain;
                    }
                }
                self.preset = Some(preset_name.to_string());
                self.enabled = true;
                return;
            }
        }
    }

    /// Builds the ffmpeg equalizer filter string from active bands.
    pub fn build_filter(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }

        let mut filters = Vec::new();
        for band in &self.bands {
            if band.gain.abs() > 0.01 {
                filters.push(format!(
                    "equalizer=f={}:width_type=h:w={}:g={}",
                    band.frequency, band.q * band.frequency * 0.5, band.gain
                ));
            }
        }

        if filters.is_empty() {
            None
        } else {
            Some(filters.join(","))
        }
    }

    /// Applies the equalizer settings to an audio file.
    pub fn apply_to_file(&self, input: &str, output: &str) -> Result<(), String> {
        if let Some(filter) = self.build_filter() {
            crate::utils::run_ffmpeg_af(input, output, &filter)?;
            Ok(())
        } else {
            std::fs::copy(input, output)
                .map_err(|e| format!("Copy failed: {}", e))?;
            Ok(())
        }
    }
}

/// Returns all EQ presets as a convenience function.
pub fn get_eq_presets() -> &'static Vec<(&'static str, Vec<f64>)> {
    Equalizer::get_presets()
}
