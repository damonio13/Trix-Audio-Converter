/**
 * Trix Audio Converter � useSettings
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * Persistent settings hook
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useCallback } from 'react';
import { AppSettings } from '@/types';
import { usePersistedState } from './usePersistedState';

/**
 * Factory-default settings applied on first launch and on "Reset Settings".
 * These values are intentionally conservative: same-folder output, `_trix` suffix,
 * 44.1 kHz stereo, 192 kbps, zero volume adjustment, no trim, no codec-copy.
 */
const DEFAULT_SETTINGS: AppSettings = {
  defaultFormat: '',
  defaultFormats: [],
  defaultSampleRate: '44100',
  defaultChannels: '2',
  defaultBitrate: '192',
  outputDirectory: '',
  outputInSameFolder: true,
  outputSuffix: '_trix',
  quality: 100,
  volume: 0,

  codecCopy: false,
  trimStart: 0,
  trimEnd: 0,
};

/**
 * Provides persistent application settings backed by `localStorage`.
 *
 * - `settings`        — The current {@link AppSettings} object.
 * - `updateSettings`  — Merges a partial update into the current settings.
 * - `resetSettings`   — Restores all settings to {@link DEFAULT_SETTINGS}.
 */
export function useSettings() {
  const [settings, setSettings] = usePersistedState<AppSettings>('settings', DEFAULT_SETTINGS);

  /** Merges `updates` into the current settings without replacing unchanged fields. */
  const updateSettings = useCallback((updates: Partial<AppSettings>) => {
    setSettings(prev => ({ ...prev, ...updates }));
  }, [setSettings]);

  /** Restores all settings to the factory defaults. */
  const resetSettings = useCallback(() => {
    setSettings(DEFAULT_SETTINGS);
  }, [setSettings]);

  return { settings, updateSettings, resetSettings };
}
