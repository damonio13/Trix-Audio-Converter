//! Command-Line Interface (CLI)
//!
//! Provides a CLI interface for audio conversion without the GUI.
//! Built with clap for argument parsing and subcommands.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

use clap::Parser;
use crate::converter::AudioConverter;
use crate::effects::AudioEffects;
use crate::formats::OUTPUT_FORMATS;
use crate::scanner::FileScanner;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "trix", about = "Trix Audio Converter - Conversor de audio via CLI")]
struct Cli {
    #[arg(short, long, num_args = 1.., help = "Arquivos ou pastas de entrada")]
    input: Option<Vec<String>>,

    #[arg(short, long, default_value = "./output", help = "Pasta de saida")]
    output: String,

    #[arg(short, long, default_value = "mp3", help = "Formato de saida")]
    format: String,

    #[arg(short, long, default_value = "normal", help = "Preset de velocidade")]
    speed: String,

    #[arg(long, default_value = "Original", help = "Sample rate de saida")]
    sample_rate: String,

    #[arg(long, default_value = "Original", help = "Canais de saida")]
    channels: String,

    #[arg(long, default_value_t = 0, help = "Ajuste de volume em dB (-20 a +20)")]
    volume: i32,

    #[arg(long, default_value_t = 0.0, help = "Inicio do trim em segundos")]
    trim_start: f64,

    #[arg(long, default_value_t = 0.0, help = "Fim do trim (0=fim)")]
    trim_end: f64,

    #[arg(long, help = "Normalizar audio (loudnorm)")]
    normalize: bool,

    #[arg(long, default_value_t = -16, help = "Target LUFS para normalizacao")]
    lufs: i32,

    #[arg(long, help = "Nao preservar metadados")]
    no_metadata: bool,

    #[arg(long, help = "Codec copy quando possivel")]
    codec_copy: bool,

    #[arg(long, help = "Nao sobrescrever existentes")]
    no_overwrite: bool,

    #[arg(long, default_value = "", help = "Padrao de nome: {name}, {n}, {format}")]
    pattern: String,

    #[arg(long, default_value_t = 0, help = "Numero de threads (0=auto)")]
    threads: usize,

    #[arg(long, help = "Criar subpasta por formato")]
    output_subfolder: bool,

    #[arg(long, default_value_t = 0.0, help = "Tamanho maximo de saida em MB")]
    max_size: f64,

    #[arg(long, default_value_t = 0, help = "Bass boost em dB")]
    bass_boost: i32,

    #[arg(long, default_value_t = 0, help = "Treble boost em dB")]
    treble_boost: i32,

    #[arg(long, default_value_t = 0, help = "Reverb intensity (0-100)")]
    reverb: i32,

    #[arg(long, default_value_t = 1.0, help = "Velocidade do audio (0.5-2.0)")]
    speed_fx: f64,

    #[arg(long, default_value_t = 0, help = "Pitch em semitones")]
    pitch: i32,

    #[arg(long, help = "Ativar compressor")]
    compressor: bool,

    #[arg(long, help = "Ativar chorus")]
    chorus: bool,

    #[arg(long, help = "Ativar flanger")]
    flanger: bool,

    #[arg(long, default_value = "", help = "Aplicar preset de efeito")]
    effect_preset: String,

    #[arg(long, default_value = "none", help = "Acao pos-conversao")]
    post_action: String,

    #[arg(long, help = "Listar formatos disponiveis")]
    list_formats: bool,

    #[arg(long, help = "Listar presets de efeito")]
    list_effects: bool,
}

/// Collects all input files from the given paths.
/// Individual files are added directly; directories are scanned recursively
/// for all supported audio formats with the given extension.
/// Paths that fail the safety check are skipped with a warning.
fn collect_input_files(input: &[String], ext: &str) -> Vec<PathBuf> {
    let mut all_files = Vec::new();
    for inp in input {
        if !crate::utils::is_safe_path(inp) {
            println!("AVISO: '{}' caminho invalido, ignorando...", inp);
            continue;
        }
        let p = PathBuf::from(inp);
        if p.is_file() {
            all_files.push(p);
        } else if p.is_dir() {
            let files = FileScanner::scan_folders(&[inp.to_string()], ext);
            for (input, _output) in &files {
                all_files.push(PathBuf::from(input));
            }
        } else {
            println!("AVISO: '{}' nao encontrado, ignorando...", inp);
        }
    }
    all_files
}

/// Builds the [`AudioEffects`] struct from the CLI flags.
///
/// First applies values from the named preset (if `--effect-preset` is set),
/// then overrides individual fields with any explicit flag values.
/// Returns the effects struct and a bool indicating whether any effect is active.
fn build_effects(cli: &Cli) -> (AudioEffects, bool) {
    let mut effects = AudioEffects::default();
    if !cli.effect_preset.is_empty() {
        let presets = crate::effects::get_effect_presets();
        if let Some(preset) = presets.get(&cli.effect_preset) {
            if let Some(v) = preset.get("bass_boost") { if let Some(n) = v.as_i64() { effects.bass_boost = n as i32; } }
            if let Some(v) = preset.get("treble_boost") { if let Some(n) = v.as_i64() { effects.treble_boost = n as i32; } }
            if let Some(v) = preset.get("reverb") { if let Some(n) = v.as_i64() { effects.reverb = n as i32; } }
            if let Some(v) = preset.get("speed") { if let Some(n) = v.as_f64() { effects.speed = n; } }
            if let Some(v) = preset.get("pitch") { if let Some(n) = v.as_i64() { effects.pitch = n as i32; } }
            if let Some(v) = preset.get("compressor") { effects.compressor = v.as_bool().unwrap_or(false); }
            if let Some(v) = preset.get("chorus") { effects.chorus = v.as_bool().unwrap_or(false); }
            if let Some(v) = preset.get("flanger") { effects.flanger = v.as_bool().unwrap_or(false); }
            if let Some(v) = preset.get("gate") { effects.gate = v.as_bool().unwrap_or(false); }
        }
    }
    if cli.bass_boost != 0 { effects.bass_boost = cli.bass_boost; }
    if cli.treble_boost != 0 { effects.treble_boost = cli.treble_boost; }
    if cli.reverb != 0 { effects.reverb = cli.reverb; }
    if (cli.speed_fx - 1.0).abs() > f64::EPSILON { effects.speed = cli.speed_fx; }
    if cli.pitch != 0 { effects.pitch = cli.pitch; }
    if cli.compressor { effects.compressor = true; }
    if cli.chorus { effects.chorus = true; }
    if cli.flanger { effects.flanger = true; }
    let has = effects.has_effects();
    (effects, has)
}

/// Builds `(input, output, format_key)` triples for each input file.
///
/// If `pattern` is set (e.g. `"{name}_{n}"`), uses it to derive output filenames.
/// Otherwise the output filename matches the source stem with the new extension.
fn build_file_pairs(
    all_files: &[PathBuf],
    output_dir: &std::path::Path,
    pattern: &str,
    format: &str,
    ext: &str,
) -> Vec<(String, String, String)> {
    all_files.iter().enumerate().map(|(i, inp)| {
        let name = inp.file_stem().unwrap_or_default().to_string_lossy();
        let out_name = if !pattern.is_empty() {
            pattern
                .replace("{name}", &name)
                .replace("{n}", &format!("{:03}", i + 1))
                .replace("{format}", format)
        } else {
            name.to_string()
        };
        let out_path = output_dir.join(format!("{}{}", out_name, ext));
        let fmt_key = if format.starts_with('.') { format.to_string() } else { format!(".{}", format) };
        (inp.to_string_lossy().to_string(), out_path.to_string_lossy().to_string(), fmt_key)
    }).collect()
}

/// Renders an in-place terminal progress bar for the active conversion batch.
/// Uses Unicode block characters (█/░) and writes to stdout without a newline
/// so the line is updated in place on each call.
fn print_progress(status: &crate::converter::ConversionStatus) {
    let bar_len = 30;
    let filled = (bar_len as f64 * status.progress as f64 / 100.0) as usize;
    let bar = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(bar_len - filled);
    let mins = status.eta as u32 / 60;
    let secs = status.eta as u32 % 60;
    print!("\r  [{}] {:3}%  {}/{}  ETA {:02}:{:02}", bar, status.progress, status.converted, status.total, mins, secs);
    let _ = std::io::stdout().flush();
}

/// Prints a multi-line conversion summary after the batch completes.
fn print_summary(status: &crate::converter::ConversionStatus, output_dir: &std::path::Path) {
    println!("\n\n  Conversao concluida!");
    println!("  Convertidos: {}/{}", status.converted, status.total);
    if status.failed > 0 {
        println!("  Falhas:      {}", status.failed);
    }
    let elapsed = status.elapsed;
    let mins = elapsed as u32 / 60;
    let secs = elapsed as u32 % 60;
    println!("  Tempo total: {:02}:{:02}", mins, secs);
    println!("  Saida:       {}\n", output_dir.canonicalize().unwrap_or_else(|_| output_dir.to_path_buf()).display());
}

/// Runs the CLI with the given arguments, dispatching to the appropriate subcommand.
pub async fn run(args: &[String]) {
    let cli = Cli::parse_from(args);

    if cli.list_formats {
        list_formats();
        return;
    }

    if cli.list_effects {
        list_effects();
        return;
    }

    let input = match &cli.input {
        Some(i) => i.clone(),
        None => {
            println!("Uso: trix -i <pastas> -o <saida> -f <formato>");
            return;
        }
    };

    if !OUTPUT_FORMATS.contains_key(cli.format.as_str()) {
        println!("ERRO: Formato '{}' nao encontrado.", cli.format);
        println!("Use --list-formats para ver opcoes disponiveis.");
        return;
    }

    let output_dir = PathBuf::from(&cli.output);
    if let Some(dir_str) = output_dir.to_str() {
        if !crate::utils::is_safe_path(dir_str) {
            println!("ERRO: Caminho de saida invalido.");
            return;
        }
    }
    let _ = std::fs::create_dir_all(&output_dir);

    let ext = OUTPUT_FORMATS[cli.format.as_str()].ext;
    let all_files = collect_input_files(&input, &ext);

    if all_files.is_empty() {
        println!("Nenhum arquivo compativel encontrado.");
        return;
    }

    let (effects, has_effects) = build_effects(&cli);

    println!("\n  Arquivos encontrados: {}", all_files.len());
    println!("  Formato de saida:    {} ({})", cli.format, ext);
    println!("  Velocidade:          {}", cli.speed);
    println!("  Threads:             {}", if cli.threads > 0 { cli.threads.to_string() } else { "auto".into() });
    if has_effects {
        println!("  Efeitos:             {}", effects.build_filters().join(", "));
    }
    println!();

    let converter = AudioConverter::new(cli.threads);
    let file_pairs = build_file_pairs(&all_files, &output_dir, &cli.pattern, &cli.format, &ext);

    println!("  Iniciando conversao de {} arquivo(s)...\n", file_pairs.len());

    let started = converter.start(
        file_pairs,
        &cli.sample_rate,
        &cli.channels,
        cli.volume,
        cli.trim_start,
        cli.trim_end,
        &cli.pattern,
        if cli.codec_copy { "force" } else { "auto" },
        if has_effects { Some(&effects) } else { None },
        cli.output_subfolder,
        cli.max_size,
        "",
    );

    if !started {
        println!("ERRO: Nao foi possivel iniciar a conversao.");
        return;
    }

    loop {
        let status = converter.get_status();
        print_progress(&status);
        if !status.converting {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    let final_status = converter.get_status();
    print_summary(&final_status, &output_dir);
}

/// Prints all 106 supported output formats with their extension and description.
fn list_formats() {
    println!("\n  Formatos de Saida Disponiveis\n");
    for (key, info) in OUTPUT_FORMATS.iter() {
        println!("  {:30}  {:6}  {}", key, info.ext, info.desc);
    }
    println!("\n  Total: {} formatos", OUTPUT_FORMATS.len());
}

/// Prints all registered audio effect presets with their parameter values.
fn list_effects() {
    let presets = crate::effects::get_effect_presets();
    println!("\n  Presets de Efeito Disponiveis\n");
    for (name, settings) in presets {
        if name == "none" { continue; }
        let desc: Vec<String> = settings.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        println!("  {:16}  {}", name, desc.join(", "));
    }
    println!("\n  Total: {} presets", presets.len().saturating_sub(1));
}
