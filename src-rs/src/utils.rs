//! Shared utility functions: path handling, string processing
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use std::process::Command;

/// Runs a command and returns an error if it fails.
pub fn run_cmd_checked(cmd: &mut Command) -> Result<std::process::Output, String> {
    let out = cmd.output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(out)
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

/// Formats seconds into mm:ss.ms string.
pub fn format_mm_ss_ms(seconds: f64, ms_digits: u32) -> String {
    let min = (seconds / 60.0) as u32;
    let sec = (seconds % 60.0) as u32;
    let ms = (seconds.fract() * 10f64.powi(ms_digits as i32)).round() as u32;
    let width = ms_digits as usize;
    format!("{:02}:{:02}.{:0width$}", min, sec, ms, width = width)
}

/// Measures peak and RMS levels in dB for an audio file.
pub fn measure_peak_rms_db(path: &str) -> Result<(f64, f64), String> {
    let (peak_lin, rms_lin) = measure_peak_rms(path)?;
    let peak = if peak_lin > 0.0 { 20.0 * peak_lin.log10() } else { -100.0 };
    let rms = if rms_lin > 0.0 { 20.0 * rms_lin.log10() } else { -100.0 };
    Ok((peak, rms))
}

/// Supported audio file extensions.
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "wav", "mp3", "flac", "aac", "ogg", "opus", "m4a", "wma", "aiff", "alac",
];

/// Lista todos os arquivos de audio (por extensao) em um diretorio (nao recursivo).
pub fn list_audio_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_file())
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                        .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// loudnorm JSON parsing
// ---------------------------------------------------------------------------

/// Fields returned by FFmpeg's `loudnorm` filter in JSON mode.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoudnormResult {
    /// Measured integrated loudness of the input (LUFS).
    pub input_i: f64,
    /// Measured true-peak level of the input (dBTP).
    pub input_tp: f64,
    /// Measured loudness range of the input (LU).
    pub input_lra: f64,
    /// Measured loudness threshold of the input (LUFS).
    pub input_thresh: f64,
    /// Required gain offset to reach the target loudness (dB).
    pub target_offset: f64,
    /// Target integrated loudness (LUFS), typically −23.0 or −14.0.
    pub target_i: f64,
    /// Target true-peak ceiling (dBTP).
    pub target_tp: f64,
}

/// Extrai o JSON impresso pelo filtro loudnorm (print_format=json) do stderr
/// e devolve os campos medidos.
pub fn parse_loudnorm_json(stderr: &str) -> Result<LoudnormResult, String> {
    let json_start = stderr.rfind('{').ok_or("No JSON found in loudnorm output")?;
    let json_end = stderr.rfind('}').ok_or("No closing JSON brace")?;
    let json_str = &stderr[json_start..=json_end];
    let v: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| e.to_string())?;

    let input_i = v["input_i"].as_str().and_then(|s| s.parse().ok()).unwrap_or(-99.0);
    let input_tp = v["input_tp"].as_str().and_then(|s| s.parse().ok()).unwrap_or(-99.0);
    let input_lra = v["input_lra"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let input_thresh = v["input_thresh"].as_str().and_then(|s| s.parse().ok()).unwrap_or(-99.0);
    let target_offset = v["target_offset"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let target_i = v["target_i"].as_str().and_then(|s| s.parse().ok()).unwrap_or(-16.0);
    let target_tp = v["target_tp"].as_str().and_then(|s| s.parse().ok()).unwrap_or(-1.5);

    Ok(LoudnormResult { input_i, input_tp, input_lra, input_thresh, target_offset, target_i, target_tp })
}

// ---------------------------------------------------------------------------
// ffmpeg helpers
// ---------------------------------------------------------------------------

/// Executa ffmpeg com filtro -af e retorna erro se falhar.
pub fn run_ffmpeg_af(input: &str, output: &str, af: &str) -> Result<String, String> {
    let out = Command::new("ffmpeg")
        .args(["-hide_banner", "-i", input, "-af", af, "-y", "--", output])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok("Sucesso".into())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

/// Executa ffmpeg com filtro -af e retorna o stderr (para analise de metadados).
pub fn run_ffmpeg_af_stderr(input: &str, af: &str) -> Result<String, String> {
    let out = Command::new("ffmpeg")
        .args(["-hide_banner", "-i", input, "-af", af, "-f", "null", "-"])
        .output()
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&out.stderr).to_string())
}

/// Executa ffmpeg com argumentos arbitrarios e retorna o Output.
pub fn run_ffmpeg_raw(args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("ffmpeg")
        .args(args)
        .output()
        .map_err(|e| e.to_string())
}

/// Executa ffmpeg com argumentos arbitrarios e retorna Ok(stdout) ou Err(stderr).
pub fn run_ffmpeg_raw_checked(args: &[&str]) -> Result<String, String> {
    let out = run_ffmpeg_raw(args)?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

/// Retorna o JSON completo do ffprobe para um arquivo.
pub fn ffprobe_json(path: &str) -> Result<serde_json::Value, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            "--",
            path,
        ])
        .output()
        .map_err(|e| format!("ffprobe falhou: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).chars().take(200).collect());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).map_err(|e| format!("Falha ao parsear JSON: {}", e))
}

/// Retorna a duracao em segundos de um arquivo de audio.
pub fn ffprobe_duration(path: &str) -> Result<f64, String> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            "--",
            path,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    let duration_str = String::from_utf8_lossy(&output.stdout);
    duration_str.trim().parse::<f64>().map_err(|e| e.to_string())
}

/// Extrai um valor numerico (em dB) de um campo do output do ffmpeg astats.
pub fn extract_stat(text: &str, key: &str) -> Option<f64> {
    for line in text.lines() {
        if line.contains(key) {
            if let Some(val) = line.split(':').last() {
                return val.trim().replace("dB", "").parse().ok();
            }
        }
    }
    None
}

/// Verifica se um caminho e seguro (sem traversal, sem caracteres perigosos).
pub fn is_safe_path(s: &str) -> bool {
    if s.is_empty() || s.starts_with('-') || s.contains('\0') || s.contains("..") {
        return false;
    }
    if cfg!(target_os = "windows") && (s.starts_with("\\\\") || s.starts_with("\\\\?\\")) {
        return false;
    }
    true
}

/// Verificacao avancada de traversal (inclui padroes URL-encoded).
pub fn path_has_traversal(path: &str) -> bool {
    if path == ".." || path == "." {
        return true;
    }
    let lower = path.to_lowercase();
    // Check for ".." in any path component (works with both / and \)
    let parts: Vec<&str> = path.split(|c| c == '/' || c == '\\').collect();
    for part in &parts {
        if *part == ".." {
            return true;
        }
    }
    if lower.contains("..%2f") || lower.contains("%2e%2e") || lower.contains("%252e")
        || lower.contains("%2e/") || lower.contains("/%2e")
        || lower.contains("%c0%ae") || lower.contains("%c0%af")
        || lower.contains("%252e%252e") || lower.contains("%252f")
    {
        return true;
    }
    false
}

/// Validates that a file path is safe and contains no traversal sequences.
pub fn validate_safe_path(path: &str, error_msg: &str) -> Result<(), String> {
    if !is_safe_path(path) || path_has_traversal(path) {
        return Err(error_msg.into());
    }
    Ok(())
}

/// Validates that both input and output paths are safe for file operations.
pub fn validate_input_output(input: &str, output: &str) -> Result<(), String> {
    if !is_safe_path(input) {
        return Err("Caminho de entrada invalido".into());
    }
    if !is_safe_path(output) {
        return Err("Caminho de saida invalido".into());
    }
    Ok(())
}

/// Validates parameters for a Google Drive upload operation.
pub fn validate_cloud_upload_gdrive(file_path: &str, folder_id: &str, api_key: &str) -> Result<(), String> {
    if file_path.is_empty() {
        return Err("Caminho do arquivo obrigatorio".into());
    }
    if folder_id.is_empty() || folder_id.len() > 200 || !folder_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("ID da pasta invalido".into());
    }
    if api_key.is_empty() {
        return Err("Chave de API obrigatoria".into());
    }
    validate_safe_path(file_path, "Caminho do arquivo invalido")?;
    Ok(())
}

/// Validates parameters for a Google Drive download operation.
pub fn validate_cloud_download_gdrive(file_id: &str, output_path: &str, api_key: &str) -> Result<(), String> {
    if file_id.is_empty() || file_id.len() > 200 || !file_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("ID do arquivo invalido".into());
    }
    if output_path.is_empty() {
        return Err("Caminho de saida obrigatorio".into());
    }
    if api_key.is_empty() {
        return Err("Chave de API obrigatoria".into());
    }
    validate_safe_path(output_path, "Caminho de saida invalido")?;
    Ok(())
}

fn validate_dropbox_common(file_path: &str, dropbox_path: &str, access_token: &str) -> Result<(), String> {
    if file_path.is_empty() {
        return Err("Caminho do arquivo obrigatorio".into());
    }
    if dropbox_path.is_empty() {
        return Err("Caminho Dropbox obrigatorio".into());
    }
    if access_token.is_empty() {
        return Err("Token de acesso obrigatorio".into());
    }
    validate_safe_path(file_path, "Caminho do arquivo invalido")?;
    Ok(())
}

/// Validates parameters for a Dropbox upload operation.
pub fn validate_cloud_upload_dropbox(file_path: &str, dropbox_path: &str, access_token: &str) -> Result<(), String> {
    validate_dropbox_common(file_path, dropbox_path, access_token)
}

/// Validates parameters for a Dropbox download operation.
pub fn validate_cloud_download_dropbox(file_path: &str, dropbox_path: &str, access_token: &str) -> Result<(), String> {
    validate_dropbox_common(file_path, dropbox_path, access_token)
}

/// Validates parameters for listing files in a Google Drive folder.
pub fn validate_cloud_list_gdrive(folder_id: &str, api_key: &str) -> Result<(), String> {
    if folder_id.is_empty() || folder_id.len() > 200 || !folder_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("ID da pasta invalido".into());
    }
    if api_key.is_empty() {
        return Err("Chave de API obrigatoria".into());
    }
    Ok(())
}

/// Verifica se uma string de bitrate e valida (so digitos ou digitos seguidos de k/K, nao vazia, ate 8 chars).
pub fn is_valid_bitrate_str(s: &str) -> bool {
    if s.is_empty() || s.len() > 8 {
        return false;
    }
    let has_k = s.ends_with('k') || s.ends_with('K');
    let num_part = if has_k { &s[..s.len() - 1] } else { s };
    !num_part.is_empty() && num_part.chars().all(|c| c.is_ascii_digit())
}

const INVALID_FILENAME_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*', '\0'];

/// Verifica se uma string contem caracteres invalidos para nomes de arquivo/sufixos/padroes.
pub fn has_invalid_chars(s: &str) -> bool {
    s.chars().any(|c| INVALID_FILENAME_CHARS.contains(&c))
}

/// Sanitiza um nome de arquivo removendo caracteres perigosos.
pub fn sanitize_filename(name: &str) -> String {
    let mut result = String::new();
    for c in name.chars() {
        if INVALID_FILENAME_CHARS.contains(&c) {
            result.push('_');
        } else if c.is_control() {
            continue;
        } else {
            result.push(c);
        }
    }
    let result = result.trim_matches('.').trim().to_string();
    if result.is_empty() || result == ".." {
        "Unknown".to_string()
    } else {
        result
    }
}

/// Extrai o file stem (nome sem extensao) de um caminho como String lossy.
pub fn file_stem(path: &std::path::Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

/// Extrai o file stem com fallback customizado.
pub fn file_stem_or(path: &std::path::Path, fallback: &str) -> String {
    let s = file_stem(path);
    if s.is_empty() { fallback.to_string() } else { s }
}

/// Audio metadata extracted from FFprobe's JSON output.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AudioProbeInfo {
    /// Total duration of the audio stream in seconds.
    pub duration: f64,
    /// Codec name as reported by FFprobe (e.g. `"mp3"`, `"flac"`).
    pub codec: String,
    /// Sample rate in Hz (e.g. `44100`).
    pub sample_rate: u32,
    /// Number of audio channels (1 = mono, 2 = stereo, …).
    pub channels: u32,
    /// Bitrate in bits per second.
    pub bitrate: u64,
    /// Bit depth per sample (e.g. `16`, `24`, `32`). `0` if unknown.
    pub bit_depth: u32,
}

/// Extrai metadados basicos de audio do JSON do ffprobe.
pub fn parse_audio_probe(json: &serde_json::Value) -> Result<AudioProbeInfo, String> {
    let duration = json["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let streams = json["streams"].as_array();
    let audio_stream = streams
        .and_then(|s| s.iter().find(|s| s["codec_type"].as_str() == Some("audio")));

    let codec = audio_stream
        .and_then(|s| s["codec_name"].as_str())
        .unwrap_or("unknown")
        .to_string();

    let sample_rate = audio_stream
        .and_then(|s| s["sample_rate"].as_str())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let channels = audio_stream
        .and_then(|s| s["channels"].as_u64())
        .unwrap_or(0) as u32;

    let bitrate = json["format"]["bit_rate"]
        .as_str()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let bit_depth = audio_stream
        .and_then(|s| s["bits_per_raw_sample"].as_str())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    Ok(AudioProbeInfo {
        duration,
        codec,
        sample_rate,
        channels,
        bitrate,
        bit_depth,
    })
}

/// Mede peak e RMS de um arquivo de audio usando ffmpeg astats.
/// Retorna (peak_linear, rms_linear).
pub fn measure_peak_rms(path: &str) -> Result<(f64, f64), String> {
    let stderr = run_ffmpeg_af_stderr(path, "astats=metadata=1:reset=0")?;
    let peak = extract_stat(&stderr, "Peak level dB")
        .map(|db| 10f64.powf(db / 20.0))
        .unwrap_or(0.0);
    let rms = extract_stat(&stderr, "RMS level dB")
        .map(|db| 10f64.powf(db / 20.0))
        .unwrap_or(0.0);
    Ok((peak, rms))
}

/// Monta filtros de fade (afade) para ffmpeg.
/// `total_samples` e `sample_rate` sao usados para calcular o tempo de inicio do fade-out.
/// `fade_in_sec` e `fade_out_sec` sao duracoes em segundos (0.0 = sem fade).
pub fn build_fade_filters(
    total_samples: u64,
    sample_rate: u32,
    fade_in_sec: f64,
    fade_out_sec: f64,
) -> Vec<String> {
    let mut filters = Vec::new();
    let sr = sample_rate as f64;

    if fade_in_sec > 0.0 {
        filters.push(format!("afade=t=in:st=0:d={:.3}", fade_in_sec));
    }

    if fade_out_sec > 0.0 {
        let total_dur = total_samples as f64 / sr;
        let start = (total_dur - fade_out_sec).max(0.0);
        filters.push(format!("afade=t=out:st={:.3}:d={:.3}", start, fade_out_sec));
    }

    filters
}

/// Retorna um filtro pan do ffmpeg pelo nome do modo.
pub fn pan_filter(mode: &str) -> &'static str {
    match mode {
        "stereo" => "pan=stereo|c0=c0|c1=c1",
        "mono" => "pan=mono|c0=0.5*c0+0.5*c1",
        "left" => "pan=mono|c0=c0",
        "right" => "pan=mono|c0=c1",
        "surround_51_to_stereo" => "pan=stereo|c0=c0+0.707*c2+0.707*c3|c1=c1+0.707*c2+0.707*c3",
        "stereo_to_surround_51" => "pan=5.1|c0=c0|c1=c1|c2=0|c3=0|c4=0.707*c0+0.707*c1|c5=0.707*c0+0.707*c1",
        _ => "pan=stereo|c0=c0|c1=c1",
    }
}

// ---------------------------------------------------------------------------
// Shared DSP filter builders
// ---------------------------------------------------------------------------

/// Filtro equalizador parametrico (banda unica).
pub fn build_equalizer_filter(frequency: f64, gain: f64, q: f64) -> String {
    format!("equalizer=f={}:g={}:width_type=q:w={}", frequency, gain, q)
}

/// Filtro de graves (bass boost/cut).
pub fn build_bass_filter(gain_db: f64, frequency: f64) -> String {
    format!("bass=g={}:f={}", gain_db, frequency)
}

/// Filtro de agudos (treble boost/cut).
pub fn build_treble_filter(gain_db: f64, frequency: f64) -> String {
    format!("treble=g={}:f={}", gain_db, frequency)
}

/// Filtro de compressor (compand).
pub fn build_compressor_filter(threshold: f64, ratio: f64) -> String {
    format!("compand=attacks=0.3:decays=0.8:{}:ratio={}", threshold, ratio)
}

/// Filtro de alteracao de velocidade (atempo).
pub fn build_tempo_filter(speed: f64) -> String {
    format!("atempo={}", speed)
}

/// Constrói o filtro ffmpeg para um efeito dado seu tipo e parâmetros.
/// Retorna Some(filter_string) se o tipo for conhecido, None caso contrário.
pub fn build_effect_filter(effect_type: &str, params: &std::collections::HashMap<String, f64>) -> Option<String> {
    match effect_type {
        "equalizer" => {
            let freq = params.get("frequency").unwrap_or(&1000.0);
            let gain = params.get("gain").unwrap_or(&0.0);
            let q = params.get("q").unwrap_or(&1.0);
            Some(build_equalizer_filter(*freq, *gain, *q))
        }
        "bass" => {
            let gain = params.get("gain").unwrap_or(&0.0);
            let freq = params.get("frequency").unwrap_or(&100.0);
            Some(build_bass_filter(*gain, *freq))
        }
        "treble" => {
            let gain = params.get("gain").unwrap_or(&0.0);
            let freq = params.get("frequency").unwrap_or(&3000.0);
            Some(build_treble_filter(*gain, *freq))
        }
        "reverb" => {
            let roomsize = params.get("roomsize").unwrap_or(&0.5);
            let wet = params.get("wet").unwrap_or(&0.3);
            Some(format!("freeverb=roomsize={}:wet={}", roomsize, wet))
        }
        "chorus" => {
            let delay = params.get("delay").unwrap_or(&50.0);
            let depth = params.get("depth").unwrap_or(&0.5);
            let speed = params.get("speed").unwrap_or(&0.5);
            Some(format!("chorus=delay={}:depth={}:speed={}", delay, depth, speed))
        }
        "flanger" => {
            let delay = params.get("delay").unwrap_or(&0.5);
            let depth = params.get("depth").unwrap_or(&0.5);
            Some(format!("flanger=delay={}:depth={}", delay, depth))
        }
        "phaser" => {
            let speed = params.get("speed").unwrap_or(&0.5);
            let depth = params.get("depth").unwrap_or(&0.5);
            Some(format!("aphaser=speed={}:depth={}", speed, depth))
        }
        "compand" => {
            let threshold = params.get("threshold").unwrap_or(&-20.0);
            let ratio = params.get("ratio").unwrap_or(&4.0);
            Some(build_compressor_filter(*threshold, *ratio))
        }
        "limiter" => {
            let limit = params.get("limit").unwrap_or(&-1.0);
            Some(format!("alimiter=limit={}", limit))
        }
        "highpass" => {
            let freq = params.get("frequency").unwrap_or(&200.0);
            Some(format!("highpass=f={}", freq))
        }
        "lowpass" => {
            let freq = params.get("frequency").unwrap_or(&3000.0);
            Some(format!("lowpass=f={}", freq))
        }
        "volume" => {
            let gain = params.get("gain").unwrap_or(&1.0);
            Some(format!("volume={}", gain))
        }
        "tempo" => {
            let speed = params.get("speed").unwrap_or(&1.0);
            Some(build_tempo_filter(*speed))
        }
        "pitch" => {
            let semitones = params.get("semitones").unwrap_or(&0.0);
            Some(format!("asetrate={}:{}", 44100.0 * 2.0_f64.powf(semitones / 12.0), 44100.0))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Shared format-to-codec mapping
// ---------------------------------------------------------------------------

/// FFmpeg codec name and optional default bitrate for a given output format.
pub struct CodecArgs {
    pub codec: &'static str,
    pub default_bitrate: Option<&'static str>,
}

/// Returns the FFmpeg codec arguments for the given output format key.
pub fn codec_for_format(fmt: &str) -> Option<CodecArgs> {
    match fmt {
        "m4a" | "aac" => Some(CodecArgs { codec: "aac", default_bitrate: Some("192k") }),
        "mp3" => Some(CodecArgs { codec: "libmp3lame", default_bitrate: Some("192k") }),
        "ogg" => Some(CodecArgs { codec: "libvorbis", default_bitrate: None }),
        "opus" => Some(CodecArgs { codec: "libopus", default_bitrate: None }),
        "flac" => Some(CodecArgs { codec: "flac", default_bitrate: None }),
        "wav" | "pcm" => Some(CodecArgs { codec: "pcm_s16le", default_bitrate: None }),
        "alac" => Some(CodecArgs { codec: "alac", default_bitrate: None }),
        _ => None,
    }
}

/// Appends codec and bitrate arguments to an FFmpeg argument vector for the given format.
pub fn apply_codec_args<'a>(args: &mut Vec<&'a str>, fmt: &str, bitrate: Option<&'a str>, default: Option<&'a str>) -> Result<(), String> {
    if let Some(c) = codec_for_format(fmt) {
        args.extend(["-codec:a", c.codec]);
        let effective = bitrate.or(default).or(c.default_bitrate);
        if let Some(b) = effective {
            if !is_valid_bitrate_str(b) {
                return Err(format!("Bitrate invalido: '{}'. Use apenas numeros.", b));
            }
            args.extend(["-b:a", b]);
        }
    }
    Ok(())
}

/// Serializa um objeto T em JSON pretty e grava no arquivo (atomic write via temp file + rename).
pub fn save_json<T: serde::Serialize>(path: &std::path::Path, data: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp_path, path).map_err(|e| e.to_string())
}

/// Le um arquivo JSON e deserializa em T.
pub fn load_json<T: for<'de> serde::Deserialize<'de>>(path: &std::path::Path, max_size: usize) -> Result<T, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() as usize > max_size {
        return Err("Arquivo excede tamanho maximo".into());
    }
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

/// Versão atual do schema de settings
pub const SETTINGS_SCHEMA_VERSION: u32 = 2;

/// Versioned wrapper for persisted settings, enabling schema migrations.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VersionedSettings<T> {
    /// Schema version number. Incremented on breaking settings changes.
    pub version: u32,
    /// The actual settings payload.
    pub data: T,
}

/// Salva settings com versionamento
pub fn save_versioned_json<T: serde::Serialize>(path: &std::path::Path, data: &T) -> Result<(), String> {
    let wrapper = VersionedSettings {
        version: SETTINGS_SCHEMA_VERSION,
        data,
    };
    save_json(path, &wrapper)
}

/// Carrega settings com migração automática
pub fn load_versioned_json<T: for<'de> serde::Deserialize<'de>>(
    path: &std::path::Path,
    max_size: usize,
) -> Result<T, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if meta.len() as usize > max_size {
        return Err("Arquivo excede tamanho maximo".into());
    }
    let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    
    // Tenta ler como versionado primeiro
    if let Ok(wrapper) = serde_json::from_str::<VersionedSettings<T>>(&json) {
        return Ok(migrate_settings(wrapper.version, wrapper.data));
    }
    
    // Fallback: formato antigo sem versão (v1)
    let data: T = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(migrate_settings(1, data))
}

/// Migra settings de versões antigas para a atual
fn migrate_settings<T>(from_version: u32, data: T) -> T {
    match from_version {
        1 => {
            // v1 -> v2: adicionar campos novos com defaults
            // Como T é genérico, a migração real acontece no tipo concreto
            // Aqui apenas logamos
            eprintln!("[Settings] Migrando settings da v{} para v{}", from_version, SETTINGS_SCHEMA_VERSION);
            data
        }
        v if v >= SETTINGS_SCHEMA_VERSION => data,
        _ => data,
    }
}

/// Lista todos os arquivos .json em um diretorio.
pub fn list_json_files(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let path = e.path();
                    if path.extension()?.to_string_lossy() == "json" {
                        path.file_stem().map(|s| s.to_string_lossy().to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Generic store for individual JSON files (one file per item in a directory).
pub struct JsonStore<T> {
    dir: std::path::PathBuf,
    max_size: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> JsonStore<T> {
    /// Creates a new JsonStore instance.
    pub fn new(dir: std::path::PathBuf, max_size: usize) -> Self {
        Self {
            dir,
            max_size,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Saves the data into a JSON file with a sanitized name.
    pub fn save(&self, name: &str, data: &T) -> Result<(), String> {
        let safe_name = crate::utils::sanitize_filename(name);
        let path = self.dir.join(format!("{}.json", safe_name));
        crate::utils::save_json(&path, data)
    }

    /// Loads the data from a JSON file matching the sanitized name.
    pub fn load(&self, name: &str) -> Result<T, String> {
        let safe_name = crate::utils::sanitize_filename(name);
        let path = self.dir.join(format!("{}.json", safe_name));
        crate::utils::load_json(&path, self.max_size)
    }

    /// Lists all JSON files managed by this store.
    pub fn list(&self) -> Vec<String> {
        crate::utils::list_json_files(&self.dir)
    }

    /// Loads all JSON files in the directory and returns their deserialized payloads.
    pub fn load_all(&self) -> Vec<T> {
        self.list()
            .iter()
            .filter_map(|name| self.load(name).ok())
            .collect()
    }

    /// Deletes the JSON file matching the sanitized name.
    pub fn delete(&self, name: &str) -> Result<(), String> {
        let safe_name = crate::utils::sanitize_filename(name);
        let path = self.dir.join(format!("{}.json", safe_name));
        std::fs::remove_file(path).map_err(|e| e.to_string())
    }

    /// Returns the directory path of the store.
    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }
}

/// Generic store for a single JSON file containing a Vec<T>.
pub struct JsonCollection<T> {
    file_path: std::path::PathBuf,
    max_size: usize,
    items: Vec<T>,
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> JsonCollection<T> {
    /// Creates and loads a new JsonCollection.
    pub fn new(file_path: std::path::PathBuf, max_size: usize) -> Self {
        let mut col = Self {
            file_path,
            max_size,
            items: Vec::new(),
        };
        col.load();
        col
    }

    /// Loads the JSON collection from the file path.
    pub fn load(&mut self) {
        if let Ok(items) = crate::utils::load_json(&self.file_path, self.max_size) {
            self.items = items;
        }
    }

    /// Saves the current collection items to the file path.
    pub fn save(&self) -> Result<(), String> {
        crate::utils::save_json(&self.file_path, &self.items)
    }

    /// Returns a read-only slice of the collection items.
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Returns a mutable reference to the collection items.
    pub fn items_mut(&mut self) -> &mut Vec<T> {
        &mut self.items
    }

    /// Appends a new item to the collection.
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }
}
