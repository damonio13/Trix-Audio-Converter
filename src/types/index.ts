/**
 * Trix Audio Converter � index
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * TypeScript type definitions for the application
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/** Represents a supported output audio format option. */
export type AudioFormat = {
  /** Unique identifier for the format (usually the extension). */
  id: string;
  /** Human-readable display name shown in the format picker. */
  name: string;
  /** File extension without the leading dot (e.g. `"mp3"`). */
  extension: string;
};

/** Options passed to the backend `/api/start` endpoint. */
export interface ConvertOptions {
  /** Target format key (e.g. `".mp3"`, `".flac"`). */
  format: string;
  /** Sample rate label (e.g. `"44100 Hz"`) or `"Original"` to preserve the source. */
  sampleRate: string;
  /** Channel mode label (e.g. `"Stereo"`, `"Mono"`) or `"Original"`. */
  channels: string;
  /** Bitrate override (e.g. `"320k"`). Empty string keeps the encoder default. */
  bitRate: string;
  /** Volume adjustment in decibels (−0•20 to +20). `0` = no change. */
  volume: number;
  /** When `true`, copies the audio stream without re-encoding if the format allows it. */
  codecCopy: boolean;
  /** Trim start time in seconds. `0` = no trim. */
  trimStart?: number;
  /** Trim end time in seconds. `0` = no trim. */
  trimEnd?: number;
}

/** Represents a single file in the conversion queue. */
export interface FileItem {
  /** Unique identifier (deterministic hex hash of the file path). */
  id: string;
  /** Absolute filesystem path to the source file. */
  path: string;
  /** Display name (filename with extension, no directory). */
  name: string;
  /** File size in bytes. */
  size: number;
  /** Current lifecycle state of this file in the conversion pipeline. */
  status: 'pending' | 'processing' | 'completed' | 'failed';
  /** Conversion progress as a percentage (0–100). */
  progress: number;
  /** Human-readable error message when `status` is `"failed"`. */
  error?: string;
}

/** Persistent user preferences stored in `settings.json`. */
export interface AppSettings {
  /** Primary output format key (e.g. `".mp3"`). */
  defaultFormat: string;
  /** Additional output format keys for multi-format batch conversion. */
  defaultFormats: string[];
  /** Default sample rate label (e.g. `"44100 Hz"` or `"Original"`). */
  defaultSampleRate: string;
  /** Default channel mode label (e.g. `"Stereo"` or `"Original"`). */
  defaultChannels: string;
  /** Default bitrate (e.g. `"192k"`). */
  defaultBitrate: string;
  /** Output directory path when `outputInSameFolder` is `false`. */
  outputDirectory: string;
  /** When `true`, saves output files next to the source files. */
  outputInSameFolder: boolean;
  /** Optional suffix appended to output filenames (e.g. `"_trix"`). */
  outputSuffix: string;
  /** Quality/compression slider value (0–100). */
  quality: number;
  /** Volume adjustment in decibels. */
  volume: number;
  /** Enable codec-copy mode to skip re-encoding when possible. */
  codecCopy: boolean;
  /** Trim start time in seconds. */
  trimStart: number;
  /** Trim end time in seconds. */
  trimEnd: number;
}

/** State for the application-wide modal dialog system. */
export interface ModalState {
  /** Whether the modal is currently visible. */
  open: boolean;
  /** Visual variant that controls the icon and colour scheme. */
  type: 'info' | 'confirm' | 'error' | 'success' | 'prompt';
  /** Modal title text. */
  title: string;
  /** Modal body text (may contain newlines). */
  message: string;
  /** Callback invoked when the user clicks the primary confirm button. */
  onConfirm?: () => void;
  /** When `true`, the modal body scrolls instead of expanding. */
  isScrollable?: boolean;
  /** Custom label for the primary action button (defaults to "OK"). */
  buttonLabel?: string;
  /** When `true`, hides the type icon next to the title. */
  hideIcon?: boolean;
  /** Placeholder text shown inside the prompt input field. */
  promptPlaceholder?: string;
  /** Pre-filled value for the prompt input field. */
  promptDefaultValue?: string;
  /** Callback invoked with the user's input when the prompt is confirmed. */
  onPromptConfirm?: (value: string) => void;
}
