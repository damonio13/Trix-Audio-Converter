//! CD/DVD burning functionality using system tools.
//!
//! Provides an interface to optical disc burning utilities for creating
//! audio CDs and data discs from audio file collections.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::process::Command;
use std::sync::LazyLock;

/// Burns audio CDs and manages optical disc operations.
pub struct CdBurner;

impl CdBurner {
    /// Creates an audio CD image from a list of audio files.
    pub fn create_audio_cd(
        input_files: &[String],
        output_image: &str,
        speed: u32,
    ) -> Result<String, String> {
        let mut file_args = Vec::new();
        for file in input_files {
            file_args.push(file.as_str());
        }

        let output = Command::new("cdrecord")
            .args([
                "-v",
                "speed", &speed.to_string(),
                "-dao",
                "-text",
                output_image,
            ])
            .args(&file_args)
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok("CD criado com sucesso".into())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Generates a CUE sheet file from a list of audio files with titles.
    pub fn create_cue_sheet(
        files: &[(String, String)],
        output: &str,
    ) -> Result<String, String> {
        let mut cue = "FILE \"audio.wav\" WAVE\n".to_string();
        let mut index = 1;
        let mut current_time = 0.0;

        for (file, title) in files {
            let metadata = crate::metadata::AudioMetadata::read(file)?;
            let duration = metadata.duration;
            let minutes = (current_time / 60.0) as u32;
            let seconds = (current_time % 60.0) as u32;
            let frames = ((current_time % 1.0) * 75.0) as u32;

            cue += &format!(
                "  TRACK {:02} AUDIO\n    TITLE \"{}\"\n    PERFORMER \"{}\"\n    INDEX 01 {:02}:{:02}:{:02}\n",
                index, title, metadata.artist, minutes, seconds, frames
            );

            current_time += duration;
            index += 1;
        }

        std::fs::write(output, &cue).map_err(|e| e.to_string())?;
        Ok(cue)
    }

    /// Burns an audio CD image to a physical disc.
    pub fn burn_cd(
        device: &str,
        image: &str,
        speed: u32,
    ) -> Result<String, String> {
        let output = Command::new("cdrecord")
            .args([
                "-v",
                "dev", device,
                "speed", &speed.to_string(),
                "-eject",
                image,
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok("CD gravado com sucesso".into())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Returns the list of available CD burning speeds.
    pub fn get_speeds() -> &'static [(u32, &'static str)] {
        static STATIC: LazyLock<&[(u32, &'static str)]> = LazyLock::new(|| &[
            (1, "1x (mais seguro)"),
            (2, "2x"),
            (4, "4x"),
            (8, "8x"),
            (16, "16x"),
            (24, "24x (mais rápido)"),
            (32, "32x"),
            (48, "48x"),
            (52, "52x (máximo)"),
        ]);
        &*STATIC
    }

    /// Scans for available optical disc recording devices.
    pub fn get_devices() -> Vec<String> {
        let output = Command::new("cdrecord")
            .args(["-scanbus"])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.lines()
                    .filter(|l| l.contains("Dev="))
                    .map(|l| l.to_string())
                    .collect()
            }
            Err(_) => vec!["Nenhum dispositivo encontrado".into()],
        }
    }
}
