/**
 * Trix Audio Converter � TitleBar
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * Window title bar with menu and controls
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useState, useRef, useEffect, useCallback } from 'react';
import { ThemeId, AVAILABLE_THEMES } from '@/hooks/useTheme';
import { api } from '@/utils/api';

import { AppSettings } from '@/types';

/** Props for the {@link TitleBar} component. */
interface TitleBarProps {
  /** Currently active theme id. */
  theme: ThemeId;
  /** Called when the user selects a different theme from the Themes menu. */
  onToggleTheme: (id: ThemeId) => void;
  /** BCP-47 language code of the current UI language. */
  language: string;
  /** Called when the user picks a different language. */
  onSetLanguage: (lang: string) => void;
  /** Opens the native file-picker to add audio files. */
  onAddFiles?: () => void;
  /** Clears all files from the conversion queue. */
  onClearQueue?: () => void;
  /** Opens the output destination folder in Explorer. */
  onOpenDestination?: () => void;
  /** Resets all settings to factory defaults. */
  onResetSettings?: () => void;
  /** Deletes the FFmpeg temp-cache directory. */
  onClearCache?: () => void;
  /** Shows the supported-formats modal. */
  onShowFormats?: () => void;
  /** Shows the help-guide modal. */
  onShowGuide?: () => void;
  /** Registers the right-click context-menu entry in Windows Explorer. */
  onRegisterContextMenu?: () => void;
  /** Removes the right-click context-menu entry from Windows Explorer. */
  onUnregisterContextMenu?: () => void;
  /** `false` when offline; triggers the red offline banner below the menu bar. */
  isOnline?: boolean;
  /** i18n translation helper. */
  t: (key: string) => string;
  /** Current settings (used for the quick-access bitrate/sample-rate toolbar). */
  settings: AppSettings;
  /** Merges a partial update into the current settings. */
  onSettingsChange: (updates: Partial<AppSettings>) => void;
  /** Opens a prompt-style modal that returns a user-typed string via `onConfirm`. */
  onShowPromptModal: (options: {
    title: string;
    message: string;
    placeholder?: string;
    defaultValue?: string;
    onConfirm: (val: string) => void;
  }) => void;
}

/** The 10 supported UI languages, each with a BCP-47 code, native label, and flag emoji. */
const LANGUAGES = [
  { code: 'ar',    label: 'العربية', flag: '\u{1F1F8}\u{1F1E6}' },
  { code: 'de',    label: 'Deutsch', flag: '\u{1F1E9}\u{1F1EA}' },
  { code: 'en',    label: 'English', flag: '\u{1F1FA}\u{1F1F8}' },
  { code: 'es',    label: 'Español', flag: '\u{1F1EA}\u{1F1F8}' },
  { code: 'fr',    label: 'Français', flag: '\u{1F1EB}\u{1F1F7}' },
  { code: 'hi',    label: 'हिन्दी', flag: '\u{1F1EE}\u{1F1F3}' },
  { code: 'ja',    label: '日本語', flag: '\u{1F1EF}\u{1F1F5}' },
  { code: 'pt-BR', label: 'Português (BR)', flag: '\u{1F1E7}\u{1F1F7}' },
  { code: 'ru',    label: 'Русский', flag: '\u{1F1F7}\u{1F1FA}' },
  { code: 'zh-CN', label: '中文 (简体)', flag: '\u{1F1E8}\u{1F1F3}' },
];

/** Maps a theme id to its display name shown in the Themes menu. */
const THEME_NAMES: Record<string, string> = {
  'original':  'Space Violet',
  'aurora':    'Aurora Boreal',
  'cyberpunk': 'Tokyo Cyberpunk',
  'sunset':    'Sunset Obsidian',
  'emerald':   'Emerald Forest',
};

/** Maps a theme id to its emoji icon shown next to the theme name. */
const THEME_ICONS: Record<string, string> = {
  'original':  '\u{1F30C}',
  'aurora':    '\u{1F332}',
  'cyberpunk': '\u{1F306}',
  'sunset':    '\u{1F30B}',
  'emerald':   '\u{1F33F}',
};

/** Ordered top-level menu ids used for keyboard Left/Right arrow navigation. */
const MENU_IDS = ['file', 'edit', 'tools', 'themes', 'lang', 'help'] as const;

export function TitleBar({ theme, onToggleTheme, language, onSetLanguage, onAddFiles, onClearQueue, onOpenDestination, onResetSettings, onClearCache, onShowFormats, onShowGuide, onRegisterContextMenu, onUnregisterContextMenu, isOnline = true, t, settings, onSettingsChange, onShowPromptModal }: TitleBarProps) {
  const [openMenu, setOpenMenu] = useState<string | null>(null);
  const [isMaximized, setIsMaximized] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  // Close any open dropdown when the user clicks outside the menu bar or presses Escape.
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setOpenMenu(null);
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setOpenMenu(null);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, []);

  // Detect whether the window is currently maximized so the toolbar
  // shows the correct Restore vs Maximize icon.
  useEffect(() => {
    const checkMaximized = () => {
      setIsMaximized(
        window.outerWidth >= screen.availWidth &&
        window.outerHeight >= screen.availHeight
      );
    };
    window.addEventListener('resize', checkMaximized);
    checkMaximized();
    return () => window.removeEventListener('resize', checkMaximized);
  }, []);

  // Standard bitrate presets (kbps). Used to detect when the active value is
  // user-typed ("custom") so the toolbar can display a "Custom" badge instead.
  const bitratePresets = ['64', '96', '128', '160', '192', '224', '256', '320', '500'];
  const isCustomBitrate = settings.defaultBitrate && !bitratePresets.includes(settings.defaultBitrate);

  // Standard sample-rate presets (Hz). Same custom-detection logic as bitratePresets.
  const sampleRatePresets = ['8000', '11025', '16000', '22050', '32000', '44100', '48000', '88200', '96000', '176400', '192000'];
  const isCustomSampleRate = settings.defaultSampleRate && !sampleRatePresets.includes(settings.defaultSampleRate);

  /** Toggles the named dropdown open; closes it if it is already open. */
  const toggleMenu = useCallback((id: string) => {
    setOpenMenu(prev => prev === id ? null : id);
  }, []);

  /** Closes all menus and then executes the optional action callback. */
  const run = useCallback((fn?: () => void) => {
    setOpenMenu(null);
    fn?.();
  }, []);

  /**
   * Keyboard handler for each menu trigger button.
   * - Enter / Space: toggle the targeted dropdown.
   * - Escape: close all menus.
   * - ArrowRight / ArrowLeft: cycle focus to the next/previous top-level menu.
   */
  const handleMenuTriggerKeyDown = useCallback((e: React.KeyboardEvent, menuId: string) => {
    const currentIndex = MENU_IDS.indexOf(menuId as typeof MENU_IDS[number]);
    switch (e.key) {
      case 'Enter':
      case ' ':
        e.preventDefault();
        toggleMenu(menuId);
        break;
      case 'Escape':
        e.preventDefault();
        setOpenMenu(null);
        break;
      case 'ArrowRight': {
        e.preventDefault();
        const nextId = MENU_IDS[(currentIndex + 1) % MENU_IDS.length];
        setOpenMenu(nextId);
        break;
      }
      case 'ArrowLeft': {
        e.preventDefault();
        const prevId = MENU_IDS[(currentIndex - 1 + MENU_IDS.length) % MENU_IDS.length];
        setOpenMenu(prevId);
        break;
      }
    }
  }, [toggleMenu]);

  /** Closes all menus when Escape is pressed inside an open dropdown. */
  const handleDropdownKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      setOpenMenu(null);
    }
  }, []);

  /** Sends the minimize command to the Rust backend window manager. */
  const handleMinimize = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    api.windowMinimize().catch(() => {});
  }, []);

  /** Toggles between maximized and restored window state via the Rust backend. */
  const handleMaximize = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    api.windowMaximize().catch(() => {});
  }, []);

  /** Closes the application window via the Rust backend. */
  const handleClose = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    api.windowClose().catch(() => {});
  }, []);

  /**
   * Maximizes/restores the window on double-click of the draggable title bar area.
   * Clicks on `.no-drag` elements (buttons, inputs) are intentionally ignored.
   */
  const handleMenuBarDoubleClick = useCallback((e: React.MouseEvent) => {
    const target = e.target as HTMLElement;
    if (target.closest('.no-drag')) return;
    api.windowMaximize().catch(() => {});
  }, []);

  return (
    <div className="titlebar">
      {!isOnline && (
        <div className="offline-banner" role="alert" style={{
          background: 'rgba(239, 68, 68, 0.9)',
          color: '#fff',
          padding: '4px 12px',
          fontSize: '0.7rem',
          fontWeight: 600,
          textAlign: 'center',
          letterSpacing: '0.05em',
          textTransform: 'uppercase',
          borderBottom: '1px solid rgba(255,255,255,0.1)'
        }}>
          {t('offline.banner')}
        </div>
      )}
      <nav
        className={`menu-bar${isMaximized ? ' window-maximized' : ''}`}
        onDoubleClick={handleMenuBarDoubleClick}
        aria-label={t('a11y.menu')}
      >
        <div className="menu-logo no-drag">
          <svg className="logo-icon" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" aria-hidden="true">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
          </svg>
          <span className="logo-text">Trix</span>
        </div>

        <div className="menu-items no-drag" ref={menuRef}>
          {/* File */}
          <div className={`menu-item ${openMenu === 'file' ? 'active' : ''}`}>
            <button type="button" className="menu-trigger" tabIndex={0} aria-haspopup="true" aria-expanded={openMenu === 'file'} onClick={() => toggleMenu('file')} onKeyDown={(e) => handleMenuTriggerKeyDown(e, 'file')}>{t('menu.file')}</button>
            <div className="dropdown-menu" onKeyDown={handleDropdownKeyDown}>
              <button type="button" className="dropdown-item" onClick={() => run(onAddFiles)}>{t('menu.file.addFiles')}</button>
              <button type="button" className="dropdown-item" onClick={() => run(onAddFiles)}>{t('menu.file.addFolder')}</button>
              <div className="dropdown-divider"></div>
              <button type="button" className="dropdown-item" onClick={() => run(onOpenDestination)}>{t('menu.file.selectOutput')}</button>
              <div className="dropdown-divider"></div>
              <button type="button" className="dropdown-item" onClick={() => run(onClearQueue)}>{t('menu.file.clearList')}</button>
              <div className="dropdown-divider"></div>
              <button type="button" className="dropdown-item" onClick={() => run(() => api.windowClose().catch(() => {}))}>{t('menu.file.exit')}</button>
            </div>
          </div>

          {/* Edit */}
          <div className={`menu-item ${openMenu === 'edit' ? 'active' : ''}`}>
            <button type="button" className="menu-trigger" tabIndex={0} aria-haspopup="true" aria-expanded={openMenu === 'edit'} onClick={() => toggleMenu('edit')} onKeyDown={(e) => handleMenuTriggerKeyDown(e, 'edit')}>{t('menu.edit')}</button>
            <div className="dropdown-menu" onKeyDown={handleDropdownKeyDown}>
              <button type="button" className="dropdown-item" onClick={() => { onSettingsChange({ codecCopy: !settings.codecCopy }); run(); }}>
                {settings.codecCopy ? '✓ Codec Copy' : 'Codec Copy'}
              </button>
              
              <div className="dropdown-divider"></div>
              
              {/* Bitrate Submenu */}
              <div className="dropdown-submenu-container">
                <button type="button" className="dropdown-item dropdown-submenu">{t('settings.bitrate') || 'Bitrate'}</button>
                <div className="dropdown-submenu-list">
                  {['64', '96', '128', '160', '192', '224', '256', '320', '500'].map(b => (
                    <button key={b} type="button" className="dropdown-item" onClick={() => { onSettingsChange({ defaultBitrate: b }); run(); }}>
                      {settings.defaultBitrate === b ? `✓ ${b} kbps` : `${b} kbps`}
                    </button>
                  ))}
                  <div className="dropdown-divider"></div>
                  <button type="button" className="dropdown-item" onClick={() => {
                    onShowPromptModal({
                      title: (t('settings.custom') || 'Personalizado') + " Bitrate",
                      message: t('settings.customBitratePrompt') || "Digite o valor do Bitrate personalizado em kbps (ex: 250):",
                      placeholder: "ex: 250",
                      defaultValue: settings.defaultBitrate,
                      onConfirm: (val) => {
                        if (val && val.trim() !== '') {
                          onSettingsChange({ defaultBitrate: val.trim() });
                        }
                      }
                    });
                    run();
                  }}>
                    {isCustomBitrate ? `✓ ${t('settings.custom') || 'Personalizado'}: ${settings.defaultBitrate} kbps` : `${t('settings.custom') || 'Personalizado'}...`}
                  </button>
                </div>
              </div>

              {/* Sample Rate Submenu */}
              <div className="dropdown-submenu-container">
                <button type="button" className="dropdown-item dropdown-submenu">{t('settings.sampleRate') || 'Sample Rate'}</button>
                <div className="dropdown-submenu-list">
                  {['8000', '11025', '16000', '22050', '32000', '44100', '48000', '88200', '96000', '176400', '192000'].map(r => {
                    const label = Number(r) >= 1000 ? (Number(r) / 1000) + ' kHz' : r + ' Hz';
                    return (
                      <button key={r} type="button" className="dropdown-item" onClick={() => { onSettingsChange({ defaultSampleRate: r }); run(); }}>
                        {settings.defaultSampleRate === r ? `✓ ${label}` : label}
                      </button>
                    );
                  })}
                  <div className="dropdown-divider"></div>
                  <button type="button" className="dropdown-item" onClick={() => {
                    onShowPromptModal({
                      title: (t('settings.custom') || 'Personalizado') + " Sample Rate",
                      message: t('settings.customSampleRatePrompt') || "Digite o valor do Sample Rate personalizado em Hz (ex: 44100):",
                      placeholder: "ex: 44100",
                      defaultValue: settings.defaultSampleRate,
                      onConfirm: (val) => {
                        if (val && val.trim() !== '') {
                          onSettingsChange({ defaultSampleRate: val.trim() });
                        }
                      }
                    });
                    run();
                  }}>
                    {isCustomSampleRate ? `✓ ${t('settings.custom') || 'Personalizado'}: ${settings.defaultSampleRate} Hz` : `${t('settings.custom') || 'Personalizado'}...`}
                  </button>
                </div>
              </div>

              {/* Canais Submenu */}
              <div className="dropdown-submenu-container">
                <button type="button" className="dropdown-item dropdown-submenu">{t('settings.channels') || 'Canais'}</button>
                <div className="dropdown-submenu-list">
                  {[
                    { value: '1', label: 'Mono' },
                    { value: '2', label: 'Stereo' },
                    { value: '6', label: '5.1 Surround' }
                  ].map(ch => (
                    <button key={ch.value} type="button" className="dropdown-item" onClick={() => { onSettingsChange({ defaultChannels: ch.value }); run(); }}>
                      {settings.defaultChannels === ch.value ? `✓ ${ch.label}` : ch.label}
                    </button>
                  ))}
                </div>
              </div>
              
              <div className="dropdown-divider"></div>
              <button type="button" className="dropdown-item" onClick={() => run(onResetSettings)}>{t('menu.edit.reset')}</button>
            </div>
          </div>

          {/* Tools */}
          <div className={`menu-item ${openMenu === 'tools' ? 'active' : ''}`}>
            <button type="button" className="menu-trigger" tabIndex={0} aria-haspopup="true" aria-expanded={openMenu === 'tools'} onClick={() => toggleMenu('tools')} onKeyDown={(e) => handleMenuTriggerKeyDown(e, 'tools')}>{t('menu.tools')}</button>
            <div className="dropdown-menu" onKeyDown={handleDropdownKeyDown}>
              <button type="button" className="dropdown-item" onClick={() => run(onOpenDestination)}>{t('menu.tools.openDest')}</button>
              <div className="dropdown-divider"></div>
              <button type="button" className="dropdown-item" onClick={() => run(onRegisterContextMenu)}>{t('menu.tools.registerCtx')}</button>
              <button type="button" className="dropdown-item" onClick={() => run(onUnregisterContextMenu)}>{t('menu.tools.unregisterCtx')}</button>
              <div className="dropdown-divider"></div>
              <button type="button" className="dropdown-item" onClick={() => run(onClearCache)}>{t('menu.tools.clearCache')}</button>
            </div>
          </div>

          {/* Theme */}
          <div className={`menu-item ${openMenu === 'themes' ? 'active' : ''}`}>
            <button type="button" className="menu-trigger" tabIndex={0} aria-haspopup="true" aria-expanded={openMenu === 'themes'} onClick={() => toggleMenu('themes')} onKeyDown={(e) => handleMenuTriggerKeyDown(e, 'themes')}>{t('menu.theme')}</button>
            <div className="dropdown-menu" id="dropdown-themes" onKeyDown={handleDropdownKeyDown}>
              {AVAILABLE_THEMES.map(themeItem => (
                <button
                  key={themeItem.id}
                  type="button"
                  className="dropdown-item"
                  onClick={() => { onToggleTheme(themeItem.id); setOpenMenu(null); }}
                >
                  <span style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span>{THEME_ICONS[themeItem.id]}</span>
                    <span>{THEME_NAMES[themeItem.id]}</span>
                  </span>
                  <span style={{ fontWeight: 'bold', color: 'var(--primary)' }}>
                    {theme === themeItem.id ? '\u2713' : ''}
                  </span>
                </button>
              ))}
            </div>
          </div>

          {/* Language */}
          <div className={`menu-item ${openMenu === 'lang' ? 'active' : ''}`}>
            <button type="button" className="menu-trigger" tabIndex={0} aria-haspopup="true" aria-expanded={openMenu === 'lang'} onClick={() => toggleMenu('lang')} onKeyDown={(e) => handleMenuTriggerKeyDown(e, 'lang')}>{t('menu.lang')}</button>
            <div className="dropdown-menu" id="dropdown-lang" onKeyDown={handleDropdownKeyDown}>
              {LANGUAGES.map(l => (
                <button
                  key={l.code}
                  type="button"
                  className="dropdown-item"
                  onClick={() => { onSetLanguage(l.code); setOpenMenu(null); }}
                >
                  <span style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span>{l.flag}</span>
                    <span>{l.label}</span>
                  </span>
                  <span style={{ fontWeight: 'bold', color: 'var(--primary)' }}>
                    {language === l.code ? '\u2713' : ''}
                  </span>
                </button>
              ))}
            </div>
          </div>

          {/* Help */}
          <div className={`menu-item ${openMenu === 'help' ? 'active' : ''}`}>
            <button type="button" className="menu-trigger" tabIndex={0} aria-haspopup="true" aria-expanded={openMenu === 'help'} onClick={() => toggleMenu('help')} onKeyDown={(e) => handleMenuTriggerKeyDown(e, 'help')}>{t('menu.help')}</button>
            <div className="dropdown-menu" onKeyDown={handleDropdownKeyDown}>
              <button type="button" className="dropdown-item" onClick={() => run(onShowFormats)}>{t('menu.help.formats')}</button>
              <div className="dropdown-divider"></div>
              <button type="button" className="dropdown-item" onClick={() => run(onShowGuide)}>{t('menu.help.guide')}</button>
            </div>
          </div>
        </div>
      </nav>

      {/* Window Controls - separated from drag region */}
      <div className="window-controls">
        <button title={t('window.minimize')} aria-label={t('window.minimize')} onClick={handleMinimize}>
          <svg aria-hidden="true" width="10" height="1" viewBox="0 0 10 1" fill="none" stroke="currentColor" strokeWidth="1.5">
            <line x1="0" y1="0.5" x2="10" y2="0.5" />
          </svg>
        </button>
        <button title={isMaximized ? t('window.restore') : t('window.maximize')} aria-label={isMaximized ? t('window.restore') : t('window.maximize')} onClick={handleMaximize}>
          {isMaximized ? (
            <svg aria-hidden="true" id="icon-restore" width="11" height="11" viewBox="0 0 11 11" fill="none" stroke="currentColor" strokeWidth="1.4">
              <rect x="3" y="0.7" width="7" height="7" rx="1" />
              <rect x="0.7" y="3" width="7" height="7" rx="1" fill="rgba(15,10,25,0.95)" stroke="currentColor" />
            </svg>
          ) : (
            <svg aria-hidden="true" id="icon-maximize" width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5">
              <rect x="0.75" y="0.75" width="8.5" height="8.5" rx="1" />
            </svg>
          )}
        </button>
        <button className="close-btn" title={t('window.close')} aria-label={t('window.close')} onClick={handleClose}>
          <svg aria-hidden="true" width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5">
            <line x1="1" y1="1" x2="9" y2="9" />
            <line x1="9" y1="1" x2="1" y2="9" />
          </svg>
        </button>
      </div>
    </div>
  );
}
