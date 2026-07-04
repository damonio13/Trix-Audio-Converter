/**
 * Trix Audio Converter � SettingsPanel
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * Settings panel with conversion options
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useRef, useEffect } from 'react';
import { AppSettings, FileItem, ModalState } from '@/types';
import { ToolSections } from '@/components/panels/ToolSections';
import { api } from '@/utils/api';

/** Props for the {@link SettingsPanel} component. */
interface SettingsPanelProps {
  /** Current application settings object. */
  settings: AppSettings;
  /** Called with a partial update to merge into current settings. */
  onSettingsChange: (updates: Partial<AppSettings>) => void;
  /** i18n translation helper. */
  t: (key: string) => string;
  /** Queue of files currently loaded (passed through to `ToolSections`). */
  files: FileItem[];
  /** Callback to imperatively open a modal dialog. */
  showModal: (title: string, message: string, type?: ModalState['type']) => void;
}


/** Standard audio sample rates in Hz, displayed as kHz labels in the UI. */
const SAMPLE_RATES = ['8000', '11025', '16000', '22050', '32000', '44100', '48000', '88200', '96000', '176400', '192000'];
/** Channel layout options with their FFmpeg numeric values. */
const CHANNELS = [
  { value: '1', label: 'Mono' },
  { value: '2', label: 'Stereo' },
  { value: '6', label: '5.1 Surround' },
];
/** Standard lossy bitrate options in kbps. */
const BITRATES = ['64', '96', '128', '160', '192', '224', '256', '320', '500'];
/** Sentinel value used in the `<select>` to signal that the user wants a free-form custom entry. */
const CUSTOM_VALUE = 'custom';

/** Props for the {@link CustomSelectOrInput} component. */
interface CustomSelectOrInputProps {
  /** Predefined option values shown in the `<select>`. */
  options: string[];
  /** Currently active value (may be a custom value not in `options`). */
  value: string;
  /** Formats an option value into a display label (e.g. `"44100"` → `"44.1 kHz"`). */
  formatOption: (v: string) => string;
  /** Placeholder text shown inside the free-form `<input>` when a custom value is active. */
  placeholder: string;
  /** Called whenever the selected or typed value changes. */
  onChange: (value: string) => void;
  /** i18n translation helper (used for the "Custom" option label). */
  t: (key: string) => string;
}

/**
 * Renders a `<select>` with a "Custom" option that, when chosen, switches
 * the element to a free-form `<input type="number">` for arbitrary values.
 * This lets users pick from a curated list **or** type any value they need.
 */
function CustomSelectOrInput({ options, value, formatOption, placeholder, onChange, t }: CustomSelectOrInputProps) {
  const isCustom = !options.includes(value);
  return isCustom ? (
    <input
      type="number"
      className="select-input"
      placeholder={placeholder}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    />
  ) : (
    <select
      className="select-input"
      value={options.includes(value) ? value : CUSTOM_VALUE}
      onChange={(e) => {
        if (e.target.value === CUSTOM_VALUE) {
          onChange('');
        } else {
          onChange(e.target.value);
        }
      }}
    >
      {options.map(opt => (
        <option key={opt} value={opt}>{formatOption(opt)}</option>
      ))}
        <option value={CUSTOM_VALUE}>{t('settings.custom') || 'Personalizado'}</option>
    </select>
  );
}

export function SettingsPanel({ settings, onSettingsChange, t, files, showModal }: SettingsPanelProps) {
  const volumeRangeRef = useRef<HTMLInputElement>(null);

  /**
   * Paints the range-slider track with a gradient so the filled (left) portion
   * uses the primary theme colour, while the empty (right) portion stays muted.
   * Done via an inline `background` style because `::-webkit-slider-runnable-track`
   * cannot be targeted from JavaScript.
   */
  function updateSliderTrack(el: HTMLInputElement | null, val: number) {
    if (!el) return;
    const min = parseFloat(el.min) || 0;
    const max = parseFloat(el.max) || 100;
    const pct = ((val - min) / (max - min)) * 100;
    const style = getComputedStyle(document.body);
    const primary = style.getPropertyValue('--primary').trim() || '#a855f7';
    const primaryHover = style.getPropertyValue('--primary-hover').trim() || '#c084fc';
    el.style.background = `linear-gradient(to right, ${primary} 0%, ${primaryHover} ${pct}%, rgba(255,255,255,0.1) ${pct}%, rgba(255,255,255,0.1) 100%)`;
  }

  // Re-paint the volume slider track whenever the volume setting changes.
  useEffect(() => {
    const vEl = volumeRangeRef.current;
    if (vEl) {
      updateSliderTrack(vEl, settings.volume || 0);
    }
  }, [settings.volume]);

  // Block all pointer/touch events from bubbling out of the volume slider.
  // Without this, a drag on the slider can accidentally scroll or drag
  // the parent panel, causing a jarring UX on touch screens and trackpads.
  useEffect(() => {
    const vEl = volumeRangeRef.current;
    const stopProp = (e: Event) => e.stopPropagation();
    const events = ['mousedown', 'mousemove', 'mouseup', 'pointerdown', 'pointermove', 'pointerup', 'touchstart', 'touchmove'];
    
    if (vEl) events.forEach(evt => vEl.addEventListener(evt, stopProp, true));

    return () => {
      if (vEl) events.forEach(evt => vEl.removeEventListener(evt, stopProp, true));
    };
  }, []);

  return (
    <>
      {/* Destination - EXACTLY like Image Converter */}
      <label className="group-label">{t('settings.destination') || 'Destino de Salvamento'}</label>
      <div className="destination-setting">
        <label className="destination-checkbox-label">
          <input
            type="checkbox"
            checked={settings.outputInSameFolder}
            onChange={(e) => onSettingsChange({ outputInSameFolder: e.target.checked })}
          />
          <span>{t('settings.saveSameFolder') || 'Salvar na mesma pasta dos arquivos originais'}</span>
        </label>
        <label className="destination-checkbox-label" style={{ marginTop: '10px' }}>
          <input
            type="checkbox"
            checked={!!settings.outputSuffix}
            onChange={(e) => onSettingsChange({ outputSuffix: e.target.checked ? '_trix' : '' })}
          />
          <span>{t('settings.useSuffix') || "Adicionar sufixo '_trix' ao nome do arquivo"}</span>
        </label>
        {!settings.outputInSameFolder && (
          <div className="output-path-row">
            <button
              type="button"
              className="btn-browse"
              onClick={async () => {
                try {
                  const data = await api.request<{ path?: string }>('/open-folder', {
                    method: 'POST',
                    body: JSON.stringify({ mode: 'select' }),
                  });
                  if (data.path) onSettingsChange({ outputDirectory: data.path });
                } catch { /* folder select cancelled */ }
              }}
            >
              {t('settings.selectFolder') || 'Selecionar Pasta'}
            </button>
            <input
              type="text"
              className="output-path-input"
              placeholder={t('settings.folderPlaceholder') || 'Cole ou digite o caminho da pasta...'}
              value={settings.outputDirectory}
              onChange={(e) => onSettingsChange({ outputDirectory: e.target.value })}
              autoComplete="off"
              spellCheck="false"
            />
          </div>
        )}
      </div>

      {/* Audio Settings Grid - Audio specific */}
      <label className="group-label">{t('settings.audio') || 'Áudio'}</label>
      <div className="audio-settings-grid">
        <div className="control-group">
          <label className="group-label">{t('settings.sampleRate') || 'Sample Rate'}</label>
          <CustomSelectOrInput
            options={SAMPLE_RATES}
            value={settings.defaultSampleRate}
            formatOption={(r) => Number(r) >= 1000 ? (Number(r) / 1000) + ' kHz' : r + ' Hz'}
            placeholder="48000"
            onChange={(v) => onSettingsChange({ defaultSampleRate: v })}
            t={t}
          />
        </div>
        <div className="control-group">
          <label className="group-label">{t('settings.channels') || 'Canais'}</label>
          <select
            className="select-input"
            value={settings.defaultChannels}
            onChange={(e) => onSettingsChange({ defaultChannels: e.target.value })}
          >
            {CHANNELS.map(ch => (
              <option key={ch.value} value={ch.value}>{ch.label}</option>
            ))}
          </select>
        </div>
        <div className="control-group">
          <label className="group-label">{t('settings.bitrate') || 'Bitrate'}</label>
          <CustomSelectOrInput
            options={BITRATES}
            value={settings.defaultBitrate}
            formatOption={(b) => `${b} kbps`}
            placeholder="320"
            onChange={(v) => onSettingsChange({ defaultBitrate: v })}
            t={t}
          />
        </div>
      </div>

      {/* Volume - Audio specific */}
      <div className="control-group">
        <div className="label-header">
          <label className="group-label" id="volume-label">{t('settings.volume') || 'Volume'}</label>
          <span className="badge" id="volume-val">{settings.volume || 0} dB</span>
        </div>
        <input
          type="range"
          className="range-slider"
          ref={volumeRangeRef}
          min="-20"
          max="20"
          value={settings.volume || 0}
          aria-labelledby="volume-label"
          onChange={(e) => {
            const val = parseInt(e.target.value);
            onSettingsChange({ volume: val });
            updateSliderTrack(volumeRangeRef.current, val);
          }}
        />
        <div className="slider-labels">
          <span>-20 dB</span>
          <span>+20 dB</span>
        </div>
      </div>

      {/* Tool Sections */}
      <ToolSections files={files} showModal={showModal} t={t} settings={settings} onSettingsChange={onSettingsChange} />
    </>
  );
}