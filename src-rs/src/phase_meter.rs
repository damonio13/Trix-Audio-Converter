//! Stereo phase correlation meter.
//!
//! Measures the phase correlation between left and right channels
//! to detect mono compatibility issues and stereo imaging problems.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use serde::{Deserialize, Serialize};

/// Analysis result of stereo phase correlation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseAnalysis {
    pub correlation: f64,
    pub stereo_width: f64,
    pub mono_compatibility: f64,
    pub phase_issues: Vec<String>,
    pub l_r_balance: f64,
}

/// Stereo phase correlation meter.
pub struct PhaseMeter;

impl PhaseMeter {
    /// Analyzes the phase correlation of an audio file.
    pub fn analyze(input: &str) -> Result<PhaseAnalysis, String> {
        let stderr = crate::utils::run_ffmpeg_af_stderr(input, "stereotools=mlev=1.0:slev=1.0")?;

        let correlation = crate::utils::extract_stat(&stderr, "correlation").unwrap_or(0.0);
        let stereo_width = (1.0 - correlation.abs()).max(0.0);
        let mono_compatibility = (1.0 + correlation) / 2.0;

        let mut phase_issues = Vec::new();
        if correlation < 0.0 {
            phase_issues.push("Problema de fase detectado (correlação negativa)".into());
        }
        if correlation.abs() > 0.95 {
            phase_issues.push("Áudio quase mono (pouca separação estéreo)".into());
        }
        if stereo_width < 0.1 {
            phase_issues.push("Largura estéreo muito pequena".into());
        }

        Ok(PhaseAnalysis {
            correlation,
            stereo_width,
            mono_compatibility,
            phase_issues,
            l_r_balance: correlation,
        })
    }

    /// Fixes phase issues in an audio file.
    pub fn fix_phase(input: &str, output: &str) -> Result<String, String> {
        let af = "stereotools=mlev=1.0:slev=1.0,pan=stereo|c0=c0|c1=c1";
        crate::utils::run_ffmpeg_af(input, output, af)
    }

    /// Applies stereo width enhancement using FFmpeg's extrastereo filter.
    /// width controls the separation amount (1.0 = natural, >1 = wider).
    pub fn enhance_stereo(input: &str, output: &str, width: f64) -> Result<String, String> {
        let af = format!("stereotools=slev={}", width);
        crate::utils::run_ffmpeg_af(input, output, &af)
    }
}
