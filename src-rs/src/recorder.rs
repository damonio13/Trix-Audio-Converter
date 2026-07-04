//! Audio recording from system input/microphone
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Configuration for audio recording.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordingConfig {
    pub device: String,
    pub format: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub bitrate: String,
    pub output_path: String,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            device: "default".into(),
            format: "mp3".into(),
            sample_rate: 44100,
            channels: 2,
            bitrate: "192k".into(),
            output_path: "./recording".into(),
        }
    }
}

/// Represents an available audio recording device.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordingDevice {
    pub name: String,
    pub is_default: bool,
}

/// Audio recorder that captures from system input or microphone.
pub struct AudioRecorder {
    recording: Arc<AtomicBool>,
    process: Option<std::process::Child>,
}

impl AudioRecorder {
    /// Creates a new AudioRecorder in idle state.
    pub fn new() -> Self {
        Self {
            recording: Arc::new(AtomicBool::new(false)),
            process: None,
        }
    }

    /// Lists available audio recording input devices on the system.
    pub fn list_devices() -> Result<Vec<RecordingDevice>, String> {
        let output = Command::new("ffmpeg")
            .args(["-f", "dshow", "-list_devices", "true", "-i", "dummy"])
            .output()
            .map_err(|e| format!("ffmpeg failed: {}", e))?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut devices = Vec::new();
        let mut is_default = true;

        for line in stderr.lines() {
            if line.contains("(audio)") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line.rfind('"') {
                        if start != end {
                            let name = line[start + 1..end].to_string();
                            devices.push(RecordingDevice {
                                name,
                                is_default,
                            });
                            is_default = false;
                        }
                    }
                }
            }
        }

        Ok(devices)
    }

    /// Starts capturing audio from the selected device with the given config.
    pub fn start_recording(&mut self, config: &RecordingConfig) -> Result<(), String> {
        if self.recording.load(Ordering::SeqCst) {
            return Err("Already recording".into());
        }

        let ext = match config.format.as_str() {
            "mp3" => "mp3",
            "wav" => "wav",
            "flac" => "flac",
            "ogg" => "ogg",
            "aac" => "m4a",
            _ => "mp3",
        };

        let output = format!("{}.{}", config.output_path, ext);
        let device_str = format!("audio={}", config.device);
        let ar_str = config.sample_rate.to_string();
        let ac_str = config.channels.to_string();

        let mut args = vec![
            "-f", "dshow",
            "-i", &device_str,
            "-ar", &ar_str,
            "-ac", &ac_str,
        ];

        if let Some(c) = crate::utils::codec_for_format(&config.format) {
            args.extend(["-codec:a", c.codec]);
            if c.default_bitrate.is_some() {
                args.extend(["-b:a", &config.bitrate]);
            }
        }

        args.extend(["-y", "--", &output]);

        self.recording.store(true, Ordering::SeqCst);

        let child = Command::new("ffmpeg")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                self.recording.store(false, Ordering::SeqCst);
                format!("Failed to start recording: {}", e)
            })?;

        self.process = Some(child);
        Ok(())
    }

    /// Stops the active recording and returns the path of the saved file.
    pub fn stop_recording(&mut self) -> Result<String, String> {
        if !self.recording.load(Ordering::SeqCst) {
            return Err("Not recording".into());
        }

        self.recording.store(false, Ordering::SeqCst);

        if let Some(ref mut child) = self.process {
            child.kill().map_err(|e| format!("Failed to stop: {}", e))?;
        }

        Ok("Recording stopped".into())
    }

    /// Returns 	rue if a recording session is currently active.
    pub fn is_recording(&self) -> bool {
        self.recording.load(Ordering::SeqCst)
    }
}

impl Drop for AudioRecorder {
    fn drop(&mut self) {
        if self.recording.load(Ordering::SeqCst) {
            let _ = self.stop_recording();
        }
    }
}
