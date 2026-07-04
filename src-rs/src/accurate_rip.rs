//! Accurate CD ripping verification using AccurateRip checksums.
//!
//! Verifies ripped audio against the AccurateRip database to ensure
//! bit-perfect extraction from compact discs.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::process::Command;

/// AccurateRip verification engine for bit-perfect CD rip validation.
pub struct AccurateRip;

impl AccurateRip {
    /// Computes the AccurateRip disc ID from a CD audio path.
    pub fn get_disc_id(cdda_path: &str) -> Result<String, String> {
        let output = Command::new("accuraterip")
            .args(["--discid", cdda_path])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(id)
        } else {
            Err("AccurateRip não disponível".into())
        }
    }

    /// Verifies a ripped track against the AccurateRip database.
    pub fn verify_rip(
        cdda_path: &str,
        rip_path: &str,
        disc_id: &str,
        track_number: u32,
    ) -> Result<RipVerification, String> {
        let output = Command::new("accuraterip")
            .args([
                "--verify",
                "--discid", disc_id,
                "--track", &track_number.to_string(),
                cdda_path,
                rip_path,
            ])
            .output()
            .map_err(|e| e.to_string())?;

        let stderr = String::from_utf8_lossy(&output.stderr);

        let accurate = stderr.contains("Accurate") && !stderr.contains("Inaccurate");
        let confidence = if let Some(pos) = stderr.find("confidence") {
            stderr[pos..].chars().filter(|c| c.is_ascii_digit()).collect::<String>()
                .parse::<u32>()
                .unwrap_or(0)
        } else {
            0
        };

        Ok(RipVerification {
            accurate,
            confidence,
            crc_match: accurate,
        })
    }

    /// Checks whether the AccurateRip tool is available on the system.
    pub fn check_available() -> bool {
        Command::new("accuraterip")
            .arg("--version")
            .output()
            .is_ok()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Result of an AccurateRip verification check.
pub struct RipVerification {
    pub accurate: bool,
    pub confidence: u32,
    pub crc_match: bool,
}
