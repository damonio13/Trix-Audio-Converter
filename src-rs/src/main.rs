//! Trix Audio Converter — Entry Point
//!
//! Initializes the application in GUI mode (tao + wry WebView) or CLI mode (clap).
//! Sets up the HTTP API server, window management, and graceful shutdown handlers.
//!
//! Developer: João Vitor de Melo <joaovmelo259@gmail.com>
//!
//! Developer: Jo�o Vitor de Melo <joaovmelo259@gmail.com>
//! Version: 1.0.0

#![allow(dead_code)]

mod accurate_rip;
mod ai_masterer;
mod ai_noise;
mod album_art;
mod api;
mod audio_analyzer;
mod audio_catalog;
mod audio_merge_split;
mod auto_tagger;
mod batch_renamer;
mod batch_rules;
mod cd_ripper;
mod channel_mixer;
mod cloud_sync;
mod cli;
mod codec_detector;
mod codec_profiles;
mod converter;
mod cue_sheet;
mod de_esser;
mod de_reverb;
mod device_presets;
mod dsd_support;
mod dsp_chain;
mod duplicate_detector;
mod dynamic_eq;
mod effects;
mod equalizer;
mod error;
mod fade_effect;
mod fft_analyzer;
mod folder_structure;
mod formats;
mod loudness;
mod logger;
mod metadata;
mod metadata_lookup;
mod multiband_compressor;
mod phase_meter;
mod playlist;
mod plugins;
mod portable;

mod preview;
mod queue_manager;
mod recorder;
mod ringtone;
mod sample_rate;
mod scanner;
mod scheduler;
mod silence_remover;
mod spatial_audio;
mod spectrogram;
mod spectral_repair;
mod stem_separator;
mod tag_editor;
mod updater;
mod utils;
mod video_extractor;
mod watch_folder;
mod waveform;
mod loudness_penalty;
mod null_test;
mod dynamic_range_meter;
mod cd_burner;
mod internet_radio;
mod gpu_acceleration;
mod crash_logger;

use std::sync::mpsc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formats_count() {
        let count = formats::OUTPUT_FORMATS.len();
        assert!(count >= 50, "Expected at least 50 formats, got {}", count);
    }

    #[test]
    fn test_equalizer_presets() {
        let presets = equalizer::Equalizer::get_presets();
        assert!(presets.len() >= 15);
    }

    #[test]
    fn test_device_presets() {
        let presets = device_presets::DevicePresets::get_all();
        assert!(presets.len() >= 20);
    }

    #[test]
    fn test_tag_info_default() {
        let tags = tag_editor::TagInfo::default();
        assert!(tags.title.is_empty());
        assert!(tags.artist.is_empty());
    }

    #[test]
    fn test_effects_default() {
        let effects = effects::AudioEffects::default();
        assert_eq!(effects.bass_boost, 0);
        assert_eq!(effects.treble_boost, 0);
        assert_eq!(effects.speed, 1.0);
    }

    #[test]
    fn test_silence_config() {
        let config = silence_remover::SilenceConfig::default();
        assert_eq!(config.threshold_db, -40.0);
    }

    #[test]
    fn test_crossfade_config() {
        let config = audio_merge_split::CrossfadeConfig::default();
        assert_eq!(config.duration, 3.0);
    }

    #[test]
    fn test_ringtone_presets() {
        let presets = ringtone::RingtoneCutter::get_presets();
        assert!(presets.len() >= 5);
    }

    #[test]
    fn test_folder_patterns() {
        let patterns = folder_structure::FolderStructure::get_patterns();
        assert!(patterns.len() >= 5);
    }

    #[test]
    fn test_auto_tagger() {
        let result = auto_tagger::AutoTagger::parse_filename("01 - Artist - Title.mp3");
        assert_eq!(result.track, Some(1));
        assert_eq!(result.artist, Some("Artist".to_string()));
        assert_eq!(result.title, Some("Title".to_string()));
    }

    #[test]
    fn test_auto_tagger_with_year() {
        let result = auto_tagger::AutoTagger::parse_filename("Artist - Title (2024).mp3");
        assert_eq!(result.year, Some("2024".to_string()));
    }

    #[test]
    fn test_sanitize_filename() {
        let sanitized = utils::sanitize_filename("Test: File <Name>");
        assert!(!sanitized.contains(':'));
        assert!(!sanitized.contains('<'));
        assert!(!sanitized.contains('>'));
    }

    #[test]
    fn test_video_extensions() {
        let exts = video_extractor::VideoExtractor::get_supported_video_extensions();
        assert!(exts.contains(&"mp4"));
        assert!(exts.contains(&"mkv"));
        assert!(exts.contains(&"avi"));
    }

    #[test]
    fn test_loudness_penalty_platforms() {
        let platforms = loudness_penalty::LoudnessPenaltyCalculator::get_platform_targets();
        assert!(platforms.len() >= 6);
    }

    #[test]
    fn test_dynamic_range_classifications() {
        let classifications = dynamic_range_meter::DynamicRangeMeter::get_classifications();
        assert!(classifications.len() >= 4);
    }

    #[test]
    fn test_cd_burner_speeds() {
        let speeds = cd_burner::CdBurner::get_speeds();
        assert!(speeds.len() >= 6);
    }

    #[test]
    fn test_radio_formats() {
        let formats = internet_radio::InternetRadio::get_formats();
        assert!(formats.len() >= 3);
    }

    #[test]
    fn test_radio_bitrates() {
        let bitrates = internet_radio::InternetRadio::get_bitrates();
        assert!(bitrates.len() >= 4);
    }

    #[test]
    fn test_gpu_acceleration_detect() {
        let info = gpu_acceleration::GpuAcceleration::detect();
        assert!(!info.devices.is_empty() || !info.has_nvenc && !info.has_qsv);
    }

    #[test]
    fn test_gpu_codecs() {
        let codecs = gpu_acceleration::GpuAcceleration::get_gpu_accelerated_codecs();
        assert!(codecs.len() >= 5);
    }

    #[test]
    fn test_codec_args_override() {
        let codec_args = vec!["-codec:a", "libmp3lame", "-b:a", "128k", "-ar", "44100", "-ac", "2"];
        let bit_rate = "320k";
        let sample_rate = "48 kHz";
        let channels = "Mono (1)";
        
        let sr_value = crate::formats::SAMPLE_RATES.get(sample_rate).copied().unwrap_or("0");
        let ch_value = crate::formats::CHANNELS.get(channels).copied().unwrap_or("0");
        
        let mut final_codec_args = Vec::new();
        let mut idx = 0;
        while idx < codec_args.len() {
            if codec_args[idx] == "-b:a" && !bit_rate.is_empty() && bit_rate != "Original" {
                final_codec_args.push("-b:a".to_string());
                final_codec_args.push(bit_rate.to_string());
                idx += 2;
            } else if codec_args[idx] == "-ar" && sr_value != "0" {
                final_codec_args.push("-ar".to_string());
                final_codec_args.push(sr_value.to_string());
                idx += 2;
            } else if codec_args[idx] == "-ac" && ch_value != "0" {
                final_codec_args.push("-ac".to_string());
                final_codec_args.push(ch_value.to_string());
                idx += 2;
            } else {
                final_codec_args.push(codec_args[idx].to_string());
                idx += 1;
            }
        }
        
        assert_eq!(final_codec_args, vec!["-codec:a", "libmp3lame", "-b:a", "320k", "-ar", "48000", "-ac", "1"]);
    }
}

fn main() {
    // Install panic hook FIRST to capture any crash
    crash_logger::install();

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] != "--gui" {
        // CLI mode: run in a dedicated tokio runtime
        let rt = tokio::runtime::Runtime::new().expect("Falha ao criar runtime Tokio");
        rt.block_on(async {
            cli::run(&args).await;
        });
        return;
    }

    // GUI mode: tao+wry webview

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder};
    use tao::window::WindowBuilder;
    use wry::WebViewBuilder;


    // Channel for window commands from API handlers
    let (tx, rx) = mpsc::channel::<String>();

    // Store sender globally so API handlers can access it
    get_window_tx().lock().unwrap_or_else(|e| e.into_inner()).replace(tx);

    // Create global converter for graceful shutdown
    let converter = Arc::new(AudioConverter::new(0));
    set_global_converter(converter.clone());

    // Shutdown flag for signal handling
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_signal = shutdown.clone();

    // Set up signal handler for Ctrl+C
    ctrlc::set_handler(move || {
        if shutdown_signal.load(Ordering::SeqCst) {
            eprintln!("[Trix] Forçando saída...");
            std::process::exit(1);
        }
        eprintln!("[Trix] Sinal de desligamento recebido, finalizando...");
        shutdown_signal.store(true, Ordering::SeqCst);
    }).expect("[Trix] Falha ao configurar handler de sinal");

    // Start a dedicated tokio runtime in a background thread for the async server.
    // IMPORTANT: We must NOT call rt.block_on() on the main thread after this point,
    // because tao's event_loop also needs the main thread and tokio panics if the
    // runtime is dropped inside an async context.
    let converter_for_server = converter.clone();
    {
        // Scope the rt so it is dropped before we enter the tao event_loop.
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Falha ao criar runtime Tokio");
        rt.spawn(async move {
            api::serve_with_port(Some(converter_for_server)).await;
        });
        // Detach: convert to background by forgetting the handle (server runs forever)
        std::mem::forget(rt);
    }

    // Wait for server to start — use plain TCP (no tokio on main thread!)
    fn tcp_port_open(port: u16) -> bool {
        use std::net::{TcpStream, SocketAddr};
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(200)).is_ok()
    }

    let port = 'server_ready: {
        for _ in 0..100 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if let Some(&p) = SERVER_PORT.get() {
                if tcp_port_open(p) {
                    break 'server_ready p;
                }
            }
        }
        eprintln!("[Trix] Servidor nao respondeu em 10s.");
        let fallback_port = SERVER_PORT.get().copied().unwrap_or(0);
        eprintln!("[Trix] Servidor rodando em http://localhost:{} - pressione Ctrl+C para parar", fallback_port);
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    };
    let api_token = API_TOKEN.get().map(|s| s.as_str()).unwrap_or("");

    // Detect if Vite dev server is running on port 8888 — plain TCP check
    let is_dev = tcp_port_open(8888);
    let url = if is_dev {
        eprintln!("[Trix] Servidor de desenvolvimento detectado na porta 8888, conectando...");
        "http://localhost:8888".to_string()
    } else {
        format!("http://localhost:{}", port)
    };

    let init_script = format!(
        "window.__API_URL = 'http://localhost:{}'; window.__API_TOKEN = '{}';",
        port, api_token
    );

    // Create tao window on main thread
    let event_loop = EventLoopBuilder::new().build();
    let _ = EVENT_LOOP_PROXY.set(event_loop.create_proxy());

    let window_icon = load_window_icon();

    let mut window_builder = WindowBuilder::new()
        .with_title("Trix Audio Converter")
        .with_maximized(true)
        .with_inner_size(tao::dpi::LogicalSize::new(1200.0, 800.0))
        .with_min_inner_size(tao::dpi::LogicalSize::new(900.0, 600.0))
        .with_position(tao::dpi::LogicalPosition::new(100.0, 100.0))
        .with_decorations(false)
        .with_visible(false); // start hidden, show after WebView ready

    if let Some(icon) = window_icon {
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    eprintln!("[Trix] Criando janela...");
    let window = match window_builder.build(&event_loop) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[Trix] Falha ao criar janela: {}", e);
            return;
        }
    };
    eprintln!("[Trix] Janela criada. Inicializando WebView...");
    let unique_data_dir = std::env::temp_dir().join(format!("trix_webview_{}", std::process::id()));
    let mut web_context = wry::WebContext::new(Some(unique_data_dir));
    let _webview = match WebViewBuilder::new_with_web_context(&mut web_context)
        .with_url(&url)
        .with_initialization_script(&init_script)
        .with_devtools(is_dev)
        .with_transparent(false)
        .with_drag_drop_handler(move |event| {
            if let wry::DragDropEvent::Drop { paths, .. } = event {
                if let Ok(mut files) = crate::DROPPED_FILES
                    .get_or_init(|| std::sync::Mutex::new(Vec::new()))
                    .lock()
                {
                    for path in &paths {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
                return true;
            }
            false
        })
        .build(&window)
    {
        Ok(wv) => {
            eprintln!("[Trix] WebView OK, mostrando janela...");

            window.set_visible(true);
            window.set_focus();

            // Use Windows API to force window onto primary monitor and to front.
            #[cfg(target_os = "windows")]
            {
                use tao::platform::windows::WindowExtWindows;
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::WindowsAndMessaging::{
                    AllowSetForegroundWindow, SetForegroundWindow, ShowWindow, SW_SHOWMAXIMIZED,
                };
                unsafe {
                    let hwnd = HWND(window.hwnd() as *mut core::ffi::c_void);
                    let pid = std::process::id();
                    let _ = AllowSetForegroundWindow(pid);
                    let _ = ShowWindow(hwnd, SW_SHOWMAXIMIZED);
                    let _ = SetForegroundWindow(hwnd);
                    eprintln!("[Trix] Janela exibida maximizada");
                }
            }

            wv
        }
        Err(e) => {
            let safe_url = format!("http://localhost:{}", port);
            let err_msg = format!("[Trix] WebView2 falhou: {}\n", e);
            eprintln!("{}", err_msg);
            let _ = std::fs::write("trix_webview_error.txt", &err_msg);
            eprintln!("[Trix] Servidor em {} - Ctrl+C para parar", safe_url);
            loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        }
    };

    fn log_debug(msg: &str) {
        use std::fs::OpenOptions;
        use std::io::Write;
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("trix_debug_log.txt") {
            let _ = writeln!(file, "[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg);
        }
    }

    log_debug("Iniciando EventLoop...");

    // Run the event loop on the main thread (no tokio context here — safe!)
    // NOTE: _webview must be captured here to stay alive for the duration of the event loop.
    // If not captured, the WebView2 is dropped immediately, closing the window.
    event_loop.run(move |event, _, control_flow| {
        let _ = &_webview; // keep WebView2 alive
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent { event: ref wevent, .. } = event {
            log_debug(&format!("WindowEvent: {:?}", wevent));
        }

        // Check for window commands from API
        if let Ok(cmd) = rx.try_recv() {
            log_debug(&format!("Comando de janela recebido: {}", cmd));
            match cmd.as_str() {
                "close" => {
                    log_debug("Comando close: saindo...");
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                "minimize" => {
                    log_debug("Comando minimize: minimizando...");
                    window.set_minimized(true);
                }
                "maximize" => {
                    log_debug("Comando maximize: alterando estado...");
                    let is_max = window.is_maximized();
                    if is_max {
                        window.set_maximized(false);
                    } else {
                        window.set_maximized(true);
                    }
                }
                _ => {}
            }
        }

        match event {
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent { event: WindowEvent::DroppedFile(ref path), .. } => {
                let path_str = path.to_string_lossy().to_string();
                if let Ok(mut files) = DROPPED_FILES.get_or_init(|| Mutex::new(Vec::new())).lock() {
                    files.push(path_str);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                log_debug("WindowEvent::CloseRequested recebido: saindo...");
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });

    // Graceful shutdown (reached when event_loop exits via ControlFlow::Exit)
    #[allow(unreachable_code)]
    {
        log_debug("EventLoop finalizado, iniciando desligamento gracioso...");
        eprintln!("[Trix] Iniciando desligamento gracioso...");
        shutdown.store(true, Ordering::SeqCst);
        converter.cancel();
        eprintln!("[Trix] Desligamento completo");
    }
}

/// Loads the embedded window icon from `assets/icons/trix_logo_sunset_icon.png`.
/// Handles RGB, RGBA, Grayscale, and GrayscaleAlpha PNG colour types.
/// Returns `None` if the PNG cannot be decoded or if tao rejects the pixel data.
fn load_window_icon() -> Option<tao::window::Icon> {
    let icon_bytes = include_bytes!("../../assets/icons/trix_logo_sunset_icon.png");
    let mut decoder = png::Decoder::new(std::io::Cursor::new(icon_bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let (width, height, color_type) = {
        let info = reader.info();
        (info.width, info.height, info.color_type)
    };
    let mut buf = vec![0u8; reader.output_buffer_size()];
    reader.next_frame(&mut buf).ok()?;
    let rgba_buf: Vec<u8> = match color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf.chunks_exact(3)
            .flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => buf.chunks_exact(2)
            .flat_map(|c| [c[0], c[0], c[0], c[1]]).collect(),
        png::ColorType::Grayscale => buf.iter()
            .flat_map(|&v| [v, v, v, 255]).collect(),
        _ => return None,
    };
    if rgba_buf.is_empty() { return None; }
    tao::window::Icon::from_rgba(rgba_buf, width, height).ok()
}

use std::sync::{Arc, Mutex, OnceLock};
use crate::converter::AudioConverter;

static WINDOW_TX: OnceLock<Mutex<Option<mpsc::Sender<String>>>> = OnceLock::new();
/// TCP port the local API server listens on.
pub static SERVER_PORT: OnceLock<u16> = OnceLock::new();
/// Bearer token used to authenticate API requests.
pub static API_TOKEN: OnceLock<String> = OnceLock::new();
/// Files dropped onto the application window, pending conversion.
pub static DROPPED_FILES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
/// The single [`AudioConverter`] instance shared between the API server and graceful shutdown.
static GLOBAL_CONVERTER: OnceLock<Arc<AudioConverter>> = OnceLock::new();
/// Proxy handle to the tao event loop, used to wake it up after sending window commands.
static EVENT_LOOP_PROXY: OnceLock<tao::event_loop::EventLoopProxy<()>> = OnceLock::new();

/// Returns the global [`AudioConverter`] if one has been registered via [`set_global_converter`].
fn get_global_converter() -> Option<Arc<AudioConverter>> {
    GLOBAL_CONVERTER.get().cloned()
}

/// Stores the global `AudioConverter` instance for API handlers and graceful shutdown.
pub fn set_global_converter(converter: Arc<AudioConverter>) {
    let _ = GLOBAL_CONVERTER.set(converter);
}

/// Returns the static window-command sender, initialising it to `None` on first access.
fn get_window_tx() -> &'static Mutex<Option<mpsc::Sender<String>>> {
    WINDOW_TX.get_or_init(|| Mutex::new(None))
}

/// Sends a window management command (`close`, `minimize`, `maximize`) to the main event loop.
pub fn send_window_cmd(cmd: &str) {
    if let Some(sender) = get_window_tx().lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        if let Err(e) = sender.send(cmd.to_string()) {
            eprintln!("[Trix] Falha ao enviar comando '{}': {}", cmd, e);
        }
        if let Some(proxy) = EVENT_LOOP_PROXY.get() {
            let _ = proxy.send_event(());
        }
    }
}
