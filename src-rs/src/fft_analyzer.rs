//! FFT-based frequency spectrum analysis.
//!
//! Performs real-time FFT analysis on audio buffers to produce
//! spectral data for visualization and diagnostic purposes.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// FFT spectrum analysis result with frequency and magnitude data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumData {
    pub frequencies: Vec<f64>,
    pub magnitudes: Vec<f64>,
    pub peaks: Vec<SpectrumPeak>,
    pub total_energy: f64,
    pub spectral_centroid: f64,
    pub spectral_rolloff: f64,
}

/// A single peak in the frequency spectrum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumPeak {
    pub frequency: f64,
    pub magnitude: f64,
}

/// Performs FFT-based frequency spectrum analysis on audio files.
pub struct FftAnalyzer;

impl FftAnalyzer {
    /// Analyzes the audio file and returns spectrum data with peaks and centroid.
    pub fn analyze(input: &str, resolution: u32) -> Result<SpectrumData, String> {
        let output = crate::utils::run_ffmpeg_raw(&[
            "-hide_banner",
            "-i", input,
            "-af", &format!("showspectrumpic=s={}x1:mode=line:color=white", resolution),
            "-frames:v", "1",
            "-f", "rawvideo",
            "-pix_fmt", "gray",
            "-",
        ])?;

        let data = &output.stdout;
        let magnitudes: Vec<f64> = data.iter()
            .map(|&b| b as f64 / 255.0)
            .collect();

        let nyquist = 22050.0;
        let bin_width = nyquist / magnitudes.len() as f64;
        let frequencies: Vec<f64> = (0..magnitudes.len())
            .map(|i| i as f64 * bin_width)
            .collect();

        let mut peaks = Vec::new();
        for i in 1..magnitudes.len() - 1 {
            if magnitudes[i] > magnitudes[i - 1] && magnitudes[i] > magnitudes[i + 1] && magnitudes[i] > 0.5 {
                peaks.push(SpectrumPeak {
                    frequency: frequencies[i],
                    magnitude: magnitudes[i],
                });
            }
        }
        peaks.sort_by(|a, b| b.magnitude.partial_cmp(&a.magnitude).unwrap());
        peaks.truncate(10);

        let total_energy: f64 = magnitudes.iter().sum();
        let weighted_sum: f64 = frequencies.iter().zip(magnitudes.iter())
            .map(|(f, m)| f * m)
            .sum();
        let spectral_centroid = if total_energy > 0.0 { weighted_sum / total_energy } else { 0.0 };

        let mut cumulative = 0.0;
        let target = total_energy * 0.85;
        let mut spectral_rolloff = nyquist;
        for (i, mag) in magnitudes.iter().enumerate() {
            cumulative += mag;
            if cumulative >= target {
                spectral_rolloff = frequencies[i];
                break;
            }
        }

        Ok(SpectrumData {
            frequencies,
            magnitudes,
            peaks,
            total_energy,
            spectral_centroid,
            spectral_rolloff,
        })
    }

    /// Returns available FFT resolution sizes with descriptions.
    pub fn get_fft_sizes() -> Vec<(u32, &'static str)> {
        static STATIC: LazyLock<&[(u32, &str)]> = LazyLock::new(|| {
            &[
                (512, "512 (rápido)"),
                (1024, "1024 (padrão)"),
                (2048, "2048 (qualidade)"),
                (4096, "4096 (alta qualidade)"),
                (8192, "8192 (máxima)"),
            ]
        });
        STATIC.to_vec()
    }
}
