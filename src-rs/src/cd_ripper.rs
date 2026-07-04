//! CD audio extraction with CDDB metadata lookup.
//!
//! Rips audio tracks from compact discs and queries online CDDB/FreeDB
//! databases for automatic track and album metadata retrieval.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Represents a single track on an audio CD.
pub struct CdTrack {
    pub number: u32,
    pub title: String,
    pub artist: String,
    pub duration: f64,
    pub start_frame: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
/// Contains metadata and track list for an audio CD.
pub struct CdInfo {
    pub disc_id: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub genre: String,
    pub tracks: Vec<CdTrack>,
}

/// Rips audio tracks from CDs with CDDB metadata lookup.
pub struct CdRipper;

impl CdRipper {
    /// Detects an audio CD in the specified drive and returns its info.
    pub fn detect_cd(drive: &str) -> Result<CdInfo, String> {
        let output = Command::new("ffmpeg")
            .args(["-f", "cdrom", "-i", drive, "-t", "0.1", "-f", "null", "-"])
            .output()
            .map_err(|e| format!("Cannot access CD drive: {}", e))?;

        let stderr = String::from_utf8_lossy(&output.stderr);

        let track_count = Self::parse_track_count(&stderr);

        if track_count == 0 {
            return Err("No audio tracks found on CD".into());
        }

        let mut tracks = Vec::new();
        for i in 1..=track_count {
            tracks.push(CdTrack {
                number: i,
                title: format!("Track {}", i),
                artist: String::new(),
                duration: 0.0,
                start_frame: 0,
            });
        }

        Ok(CdInfo {
            disc_id: String::new(),
            artist: String::new(),
            album: String::new(),
            year: String::new(),
            genre: String::new(),
            tracks,
        })
    }

    fn parse_track_count(stderr: &str) -> u32 {
        let mut max_track = 0;
        for line in stderr.lines() {
            if line.contains("Track") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, part) in parts.iter().enumerate() {
                    if *part == "Track" && i + 1 < parts.len() {
                        if let Ok(n) = parts[i + 1].trim_matches(':').parse::<u32>() {
                            if n > max_track {
                                max_track = n;
                            }
                        }
                    }
                }
            }
        }
        max_track
    }

    /// Looks up CD metadata from the CDDB/GnuDB online database.
    pub async fn lookup_cddb(disc_id: &str) -> Result<(String, String, String, Vec<String>), String> {
        let url = format!(
            "https://gnudb.gnudb.org/~cddb/cddb.cgi?cmd=cd+read&discid={}&hello=user+audiomaster+audiomaster+1.0&proto=6",
            disc_id
        );

        let client = reqwest::Client::new();
        let resp = client.get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() || e.is_timeout() {
                    "Sem conexão com a internet para consultar CDDB. Metadados do CD não disponíveis offline.".to_string()
                } else {
                    format!("Falha na consulta CDDB: {}", e)
                }
            })?;

        let body = resp.text().await
            .map_err(|e| format!("Failed to read CDDB response: {}", e))?;

        let mut artist = String::new();
        let mut album = String::new();
        let mut year = String::new();
        let mut titles = Vec::new();

        for line in body.lines() {
            if line.starts_with("DTITLE=") {
                let info = line.trim_start_matches("DTITLE=");
                if let Some(pos) = info.find('/') {
                    artist = info[..pos].trim().to_string();
                    album = info[pos + 1..].trim().to_string();
                }
            } else if line.starts_with("DYEAR=") {
                year = line.trim_start_matches("DYEAR=").to_string();
            } else if line.starts_with("TTITLE") {
                if let Some(pos) = line.find('=') {
                    let title = line[pos + 1..].trim().to_string();
                    titles.push(title);
                }
            }
        }

        Ok((artist, album, year, titles))
    }

    /// Rips a single track from a CD to the specified output format.
    pub fn rip_track(drive: &str, track: u32, output_path: &str, format: &str, bitrate: &str) -> Result<String, String> {
        let ext = match format {
            "flac" => "flac",
            "mp3" => "mp3",
            "aac" => "m4a",
            "wav" => "wav",
            "ogg" => "ogg",
            _ => "flac",
        };

        let output_file = format!("{}/track{:02}.{}", output_path, track, ext);
        let track_meta = format!("track={}", track);

        let mut args = vec![
            "-f", "cdrom",
            "-i", drive,
            "-map", "0:a",
            "-metadata", &track_meta,
        ];

        if let Some(c) = crate::utils::codec_for_format(format) {
            args.extend(["-codec:a", c.codec]);
            if c.default_bitrate.is_some() {
                args.extend(["-b:a", bitrate]);
            }
        }

        args.push("--");
        args.push(&output_file);

        let out = crate::utils::run_ffmpeg_raw(&args)
            .map_err(|e| format!("ffmpeg failed: {}", e))?;

        if out.status.success() {
            Ok(output_file)
        } else {
            Err(format!("Failed to rip track {}", track))
        }
    }

    /// Ejects the CD tray of the specified drive.
    pub fn eject(drive: &str) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let _ = drive;
            // Windows eject via powershell
            let _ = Command::new("powershell")
                .args(["-Command", "($drive = New-Object -ComObject IMAPI2.MsftDiscRecorder2; $drive.Initialize(1, $drive); $drive.EjectMedia())"])
                .status();
        }
        #[cfg(target_os = "linux")]
        {
            Command::new("eject")
                .arg(drive)
                .status()
                .map_err(|e| format!("eject failed: {}", e))?;
        }
        Ok(())
    }
}
