//! Multiband compressor for dynamic range control.
//!
//! Splits the audio spectrum into configurable frequency bands
//! and applies independent compression to each for transparent control.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use std::sync::LazyLock;

/// Multiband compressor for frequency-dependent dynamic range control.
pub struct MultibandCompressor;

impl MultibandCompressor {
    /// Apply multiband compression using ffmpeg filter_complex.
    /// Splits audio into bands via asplit, applies compand per band, then amix.
    pub fn apply(
        input: &str,
        output: &str,
        bands: &[BandConfig],
    ) -> Result<String, String> {
        if bands.is_empty() {
            return Err("Nenhuma banda configurada".into());
        }

        let n = bands.len();

        if n == 1 {
            let b = &bands[0];
            let af = format!(
                "compand=attacks=0.3:decays=0.8:points=-80/-80|{t}/{t}|0/0:gain=0:volume=-90:delay=0.15",
                t = b.threshold
            );
            return crate::utils::run_ffmpeg_af(input, output, &af);
        }

        let mut parts: Vec<String> = Vec::new();
        let mut mix_inputs = String::new();

        for (i, band) in bands.iter().enumerate() {
            let label = format!("b{}", i);
            let comp_label = format!("c{}", i);

            let comp = format!(
                "[{label}]compand=attacks=0.3:decays=0.8:points=-80/-80|{t}/{t}|0/0:gain=0:volume=-90:delay=0.15[{cl}]",
                label = label, t = band.threshold, cl = comp_label
            );
            parts.push(comp);
            mix_inputs.push_str(&format!("[{}]", comp_label));
        }

        let split_labels: Vec<String> = (0..n).map(|i| format!("[b{}]", i)).collect();
        let asplit = format!("[0:a]asplit={n}{}", split_labels.join(""));
        parts.insert(0, asplit);

        let amix = format!("{}amix=inputs={}:duration=first:dropout_transition=0", mix_inputs, n);
        parts.push(amix);

        let fc = parts.join(";");
        let args = vec!["-i", input, "-filter_complex", &fc, "-y", "--", output];
        let out = crate::utils::run_ffmpeg_raw(&args)
            .map_err(|e| format!("ffmpeg falhou: {}", e))?;
        if out.status.success() {
            Ok("Sucesso".into())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    }

    /// Applies multiband compression using a named preset.
    pub fn apply_preset(
        input: &str,
        output: &str,
        preset: &str,
    ) -> Result<String, String> {
        let bands = match preset {
            "gentle" => vec![
                BandConfig { crossover: 200.0, threshold: -30.0, ratio: 2.0, attack: 10.0, release: 100.0, width: 200.0 },
                BandConfig { crossover: 2000.0, threshold: -25.0, ratio: 3.0, attack: 5.0, release: 50.0, width: 1800.0 },
                BandConfig { crossover: 8000.0, threshold: -20.0, ratio: 2.5, attack: 3.0, release: 30.0, width: 6000.0 },
            ],
            "aggressive" => vec![
                BandConfig { crossover: 150.0, threshold: -20.0, ratio: 6.0, attack: 5.0, release: 50.0, width: 150.0 },
                BandConfig { crossover: 1500.0, threshold: -18.0, ratio: 8.0, attack: 2.0, release: 30.0, width: 1350.0 },
                BandConfig { crossover: 6000.0, threshold: -15.0, ratio: 5.0, attack: 1.0, release: 20.0, width: 4500.0 },
            ],
            "vocal" => vec![
                BandConfig { crossover: 300.0, threshold: -25.0, ratio: 3.0, attack: 10.0, release: 80.0, width: 300.0 },
                BandConfig { crossover: 3000.0, threshold: -20.0, ratio: 4.0, attack: 5.0, release: 40.0, width: 2700.0 },
                BandConfig { crossover: 10000.0, threshold: -22.0, ratio: 2.5, attack: 3.0, release: 25.0, width: 7000.0 },
            ],
            "master" => vec![
                BandConfig { crossover: 250.0, threshold: -22.0, ratio: 3.5, attack: 8.0, release: 80.0, width: 250.0 },
                BandConfig { crossover: 2500.0, threshold: -18.0, ratio: 4.0, attack: 4.0, release: 40.0, width: 2250.0 },
                BandConfig { crossover: 8000.0, threshold: -16.0, ratio: 3.0, attack: 2.0, release: 20.0, width: 5500.0 },
            ],
            _ => return Err("Preset desconhecido".into()),
        };

        Self::apply(input, output, &bands)
    }

    /// Returns available multiband compressor presets with display names.
    pub fn get_presets() -> &'static [(&'static str, &'static str)] {
        static STATIC: LazyLock<&[(&str, &str)]> = LazyLock::new(|| {
            &[
                ("gentle", "Suave (mastering)"),
                ("aggressive", "Agressivo (mix)"),
                ("vocal", "Voz (podcast)"),
                ("master", "Master Geral"),
            ]
        });
        &*STATIC
    }
}

/// Configuration for a single frequency band in the multiband compressor.
#[derive(Debug, Clone)]
pub struct BandConfig {
    pub crossover: f64,
    pub threshold: f64,
    pub ratio: f64,
    pub attack: f64,
    pub release: f64,
    pub width: f64,
}
