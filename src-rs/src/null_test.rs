//! Null test for audio comparison and phase analysis.
//!
//! Inverts the phase of one audio signal and sums it with another
//! to reveal differences between two versions of the same recording.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0
use serde::{Deserialize, Serialize};

/// Result of a null test comparison between two audio files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullTestResult {
    pub is_identical: bool,
    pub difference_level: f64,
    pub peak_difference: f64,
    pub rms_difference: f64,
    pub result_file: Option<String>,
}

/// Performs null test comparisons between two audio files.
pub struct NullTest;

impl NullTest {
    /// Compares two audio files and outputs the difference.
    pub fn compare(
        file_a: &str,
        file_b: &str,
        output_diff: &str,
    ) -> Result<NullTestResult, String> {
        let af = "[1:a]anequalizer=c0 f=1000 w=100 g=0[muted];[0:a][muted]amix=inputs=2:duration=first:dropout_transition=0,volume=2";

        let out = crate::utils::run_ffmpeg_raw(&[
            "-hide_banner",
            "-i", file_a,
            "-i", file_b,
            "-filter_complex", af,
            "-y", "--", output_diff,
        ])?;

        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).to_string());
        }

        let (peak, rms) = crate::utils::measure_peak_rms_db(output_diff)?;

        let is_identical = peak < -120.0;
        let difference_level = if peak > -120.0 { 10.0_f64.powf(peak / 20.0) } else { 0.0 };

        Ok(NullTestResult {
            is_identical,
            difference_level,
            peak_difference: peak,
            rms_difference: rms,
            result_file: if !is_identical { Some(output_diff.to_string()) } else { None },
        })
    }

    /// Runs a stereo A/B comparison via FFmpeg's mix filter and returns a status string.
    pub fn compare_a_b(file_a: &str, file_b: &str) -> Result<String, String> {
        let af = "[0:a]asplit[a1][a2];[1:a]asplit[b1][b2];[a1][b1]amix=inputs=2:duration=first:dropout_transition=0,volume=2[ab];[a2][b2]amix=inputs=2:duration=first:dropout_transition=0,volume=2,pan=mono|c0=c0-c1[diff]";

        let out = crate::utils::run_ffmpeg_raw(&[
            "-hide_banner",
            "-i", file_a,
            "-i", file_b,
            "-filter_complex", af,
            "-map", "[ab]",
            "-map", "[diff]",
            "-y", "--", "/dev/null",
        ])?;

        if out.status.success() {
            Ok("Comparação A/B concluída".into())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    }
}
