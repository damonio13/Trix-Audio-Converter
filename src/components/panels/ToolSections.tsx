/**
 * Trix Audio Converter � ToolSections
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * Accordion tool sections (trim, options, video extract)
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useState } from 'react'
import { FileItem, ModalState, AppSettings } from '@/types'
import { api } from '@/utils/api'

/** Shared props passed to every accordion section inside {@link ToolSections}. */
interface Props {
  /** Currently queued audio files (used by VideoExtractSection). */
  files: FileItem[];
  /** Imperative modal opener forwarded from App. */
  showModal: (title: string, message: string, type?: ModalState['type']) => void;
  /** i18n translation helper. */
  t: (key: string) => string;
  /** Current application settings. */
  settings: AppSettings;
  /** Partial settings updater forwarded from App. */
  onSettingsChange: (updates: Partial<AppSettings>) => void;
}

/**
 * Generic collapsible accordion section.
 * Tracks its own `open` state and renders an animated chevron indicator.
 * The `id` of the body div is derived from `title` for `aria-controls`.
 */
function Section({ title, icon, children, defaultOpen = false }: { title: string; icon: React.ReactNode; children: React.ReactNode; defaultOpen?: boolean }) {
  const [open, setOpen] = useState(defaultOpen)
  return (
    <div className="accordion-section">
      <button type="button" className="accordion-header" onClick={() => setOpen(!open)} aria-expanded={open} aria-controls={`accordion-body-${title}`}>
        <div className="accordion-header-left">
          <span className="accordion-icon">{icon}</span>
          <span className="accordion-title">{title}</span>
        </div>
        <svg className={`accordion-chevron ${open ? 'open' : ''}`} width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
          <polyline points="9 18 15 12 9 6" />
        </svg>
      </button>
      {open && <div className="accordion-body" id={`accordion-body-${title}`}>{children}</div>}
    </div>
  )
}

// ═══ Cortar / Trim ═══
/**
 * Displays two number inputs (start / end seconds) that update `trimStart`
 * and `trimEnd` in the settings. Empty input is treated as `0`.
 */
function TrimSection({ settings, onSettingsChange, t }: { settings: AppSettings; onSettingsChange: (updates: Partial<AppSettings>) => void; t: (key: string) => string }) {
  const start = settings.trimStart !== undefined ? settings.trimStart.toString() : '';
  const end = settings.trimEnd !== undefined ? settings.trimEnd.toString() : '';

  /** Parses `val` into a float and stores it as `trimStart`; empty string maps to `0`. */
  const handleStartChange = (val: string) => {
    const num = val === '' ? 0 : parseFloat(val);
    onSettingsChange({ trimStart: isNaN(num) ? 0 : num });
  };

  /** Parses `val` into a float and stores it as `trimEnd`; empty string maps to `0`. */
  const handleEndChange = (val: string) => {
    const num = val === '' ? 0 : parseFloat(val);
    onSettingsChange({ trimEnd: isNaN(num) ? 0 : num });
  };

  return (
    <div className="accordion-fields">
      <div className="accordion-field">
        <label htmlFor="trim-start">{t('trim.start')}</label>
        <input id="trim-start" type="number" className="number-input" min="0" step="0.1" placeholder="0" value={start} onChange={e => handleStartChange(e.target.value)} />
      </div>
      <div className="accordion-field">
        <label htmlFor="trim-end">{t('trim.end')}</label>
        <input id="trim-end" type="number" className="number-input" min="0" step="0.1" placeholder="0" value={end} onChange={e => handleEndChange(e.target.value)} />
      </div>
    </div>
  )
}

// ═══ Opções ═══
/**
 * Renders a single toggle for the "Codec Copy" feature.
 * When enabled, FFmpeg copies the stream without re-encoding,
 * preserving original quality and greatly reducing CPU usage.
 */
function OptionsSection({ settings, onSettingsChange, t }: { settings: AppSettings; onSettingsChange: (updates: Partial<AppSettings>) => void; t: (key: string) => string }) {
  const codecCopy = settings.codecCopy === true;

  /** Forwards the checkbox state change to the settings updater. */
  const handleCodecCopyChange = (checked: boolean) => {
    onSettingsChange({ codecCopy: checked });
  };

  return (
    <div className="accordion-options">
      <label className="accordion-toggle">
        <input type="checkbox" checked={codecCopy} onChange={e => handleCodecCopyChange(e.target.checked)} />
        <span className="accordion-toggle-slider"></span>
        <div className="accordion-toggle-info">
          <span className="accordion-toggle-label">Codec Copy</span>
          <span className="accordion-toggle-desc">{t('options.codecCopyDesc')}</span>
        </div>
      </label>
    </div>
  )
}

// ═══ Extrair Áudio de Vídeo ═══
/**
 * Lets the user choose an output format and bitrate, then calls the
 * `/video/extract` backend endpoint to demux the first queued file.
 * Lossless formats (FLAC/WAV) skip bitrate selection and use maximum quality.
 */
function VideoExtractSection({ files, showModal, t }: Props) {
  const [format, setFormat] = useState('mp3')
  const [bitrate, setBitrate] = useState('320k')
  const hasFiles = files.length > 0

  /** Triggers the extraction API call; shows a modal on success or failure. */
  const extract = async () => {
    if (!hasFiles) { showModal(t('video.warning'), t('video.warningMsg'), 'info'); return }
    try {
      const res = await api.post<{ success: boolean; error?: string }>('/video/extract', {
        input: files[0].path,
        output: files[0].path.replace(/\.[^.]+$/, `.${format}`),
        format,
        bitrate
      })
      if (res && res.success) {
        showModal(t('video.warning'), t('video.success'), 'success')
      } else {
        showModal(t('video.error'), res?.error || t('video.errorMsg'), 'error')
      }
    } catch {
      showModal(t('video.error'), t('video.errorMsg'), 'error')
    }
  }

  const isLossless = format === 'flac' || format === 'wav';

  return (
    <div className="accordion-fields">
      <div className="accordion-field">
        <label style={{ fontSize: '0.8rem', color: 'rgba(255, 255, 255, 0.7)', fontWeight: '500' }}>{t('video.format')}</label>
        <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap', marginTop: '6px' }}>
          {['aac', 'flac', 'mp3', 'ogg', 'wav'].map(f => {
            const isSelected = format === f;
            return (
              <button
                key={f}
                type="button"
                onClick={() => setFormat(f)}
                style={{
                  flex: 1,
                  padding: '8px 4px',
                  borderRadius: '6px',
                  border: isSelected ? '1px solid var(--primary)' : '1px solid rgba(255, 255, 255, 0.1)',
                  background: isSelected 
                    ? 'linear-gradient(135deg, rgba(168,85,247,0.25), rgba(236,72,153,0.15))'
                    : 'rgba(255, 255, 255, 0.03)',
                  color: isSelected ? '#ffffff' : 'rgba(255, 255, 255, 0.6)',
                  cursor: 'pointer',
                  fontWeight: isSelected ? '600' : 'normal',
                  fontSize: '0.8rem',
                  textAlign: 'center',
                  transition: 'all 0.2s ease',
                  boxShadow: isSelected ? '0 0 8px rgba(168,85,247,0.3)' : 'none',
                }}
              >
                {f.toUpperCase()}
              </button>
            );
          })}
        </div>
      </div>
      
      {!isLossless && (
        <div className="accordion-field" style={{ marginTop: '12px' }}>
          <label style={{ fontSize: '0.8rem', color: 'rgba(255, 255, 255, 0.7)', fontWeight: '500' }}>{t('video.bitrate')}</label>
          <div style={{ display: 'flex', gap: '6px', marginTop: '6px' }}>
            {['128k', '192k', '256k', '320k'].map(b => {
              const isSelected = bitrate === b;
              return (
                <button
                  key={b}
                  type="button"
                  onClick={() => setBitrate(b)}
                  style={{
                    flex: 1,
                    padding: '8px 4px',
                    borderRadius: '6px',
                    border: isSelected ? '1px solid var(--primary)' : '1px solid rgba(255, 255, 255, 0.1)',
                    background: isSelected 
                      ? 'linear-gradient(135deg, rgba(168,85,247,0.25), rgba(236,72,153,0.15))'
                      : 'rgba(255, 255, 255, 0.03)',
                    color: isSelected ? '#ffffff' : 'rgba(255, 255, 255, 0.6)',
                    cursor: 'pointer',
                    fontWeight: isSelected ? '600' : 'normal',
                    fontSize: '0.75rem',
                    textAlign: 'center',
                    transition: 'all 0.2s ease',
                    boxShadow: isSelected ? '0 0 8px rgba(168,85,247,0.3)' : 'none',
                  }}
                >
                  {b.replace('k', ' kbps')}
                </button>
              );
            })}
          </div>
        </div>
      )}

      {isLossless && (
        <div style={{ 
          fontSize: '0.75rem', 
          color: 'rgba(255, 255, 255, 0.4)', 
          marginTop: '12px', 
          fontStyle: 'italic',
          background: 'rgba(255, 255, 255, 0.02)',
          padding: '6px 10px',
          borderRadius: '6px',
          border: '1px dashed rgba(255, 255, 255, 0.05)'
        }}>
          * Formatos Lossless (sem perda) utilizam qualidade máxima automática.
        </div>
      )}

      <div className="form-footer" style={{ border: 'none', paddingTop: 0, marginTop: '16px', width: '100%' }}>
        <button 
          type="button" 
          className="btn-submit" 
          onClick={extract} 
          disabled={!hasFiles} 
          style={{ 
            width: '100%', 
            margin: 0,
            cursor: hasFiles ? 'pointer' : 'not-allowed'
          }}
        >
          {t('video.extract')}
        </button>
      </div>
    </div>
  )
}

/**
 * Returns the ordered list of accordion section descriptors,
 * including their translated titles, icon SVGs, and rendering components.
 * Called on every render so titles stay in sync with the active language.
 */
function getSections(t: (key: string) => string): { id: string; title: string; icon: React.ReactNode; Component: React.FC<Props> }[] {
  return [
    {
      id: 'trim',
      title: t('section.trim'),
      icon: (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
          <circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><line x1="20" y1="4" x2="8.12" y2="15.88"/><line x1="14.47" y1="14.48" x2="20" y2="20"/><line x1="8.12" y1="8.12" x2="12" y2="12"/>
        </svg>
      ),
      Component: TrimSection,
    },
    {
      id: 'options',
      title: t('section.options'),
      icon: (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
      ),
      Component: OptionsSection,
    },
    {
      id: 'video',
      title: t('section.video-extract'),
      icon: (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
          <polygon points="23 7 16 12 23 17 23 7"/><rect x="1" y="5" width="15" height="14" rx="2" ry="2"/>
        </svg>
      ),
      Component: VideoExtractSection,
    },
  ];
}

/**
 * Container that renders all tool accordion sections (Trim, Codec Options,
 * Video Extract) by iterating over the descriptor list returned by `getSections`.
 */
export function ToolSections({ files, showModal, t, settings, onSettingsChange }: Props) {
  const sections = getSections(t)
  return (
    <div className="accordion-container">
      {sections.map(({ id, title, icon, Component }) => (
        <Section key={id} title={title} icon={icon}>
          <Component files={files} showModal={showModal} t={t} settings={settings} onSettingsChange={onSettingsChange} />
        </Section>
      ))}
    </div>
  )
}
