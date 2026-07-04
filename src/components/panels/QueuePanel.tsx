/**
 * Trix Audio Converter � QueuePanel
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * File queue panel with drag-and-drop
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useState, useRef, useEffect } from 'react';
import { FileItem } from '@/types';
import { api } from '@/utils/api';
import { formatSize } from '@/utils/format';
import { usePickFiles } from '@/hooks/usePickFiles';

/** Props for the formats toggle sub-component. */
interface FormatsToggleProps {
  /** Whether the full format list is currently expanded. */
  showFormats: boolean;
  /** Callback to toggle the expanded state. */
  onToggle: () => void;
  /** i18n translation helper. */
  t: (key: string) => string;
  /** Sorted list of all supported extension strings (e.g. `"MP3"`, `"FLAC"`). */
  formatsList: string[];
}

/**
 * Inline collapsible list of all supported audio formats.
 * Renders a "View all / Hide" toggle link above the full comma-separated list.
 */
function FormatsToggle({ showFormats, onToggle, t, formatsList }: FormatsToggleProps) {
  return (
    <>
      <p className="formats-text" id="formats-prompt-text">
        {t('dropzone.formatsShort')}{' '}
        <span id="btn-toggle-formats" className="toggle-formats-link" onClick={(e) => { e.stopPropagation(); onToggle(); }}>
          {showFormats ? t('dropzone.hideAll') : t('dropzone.viewAll')}
        </span>
      </p>
      <div id="full-formats-list" className={`full-formats-list ${showFormats ? '' : 'hidden'}`} onClick={(e) => e.stopPropagation()}>
        {formatsList.join(', ')}
      </div>
    </>
  );
}

/** Upload arrow SVG icon used inside the dropzone empty state. */
function UploadIcon() {
  return (
    <svg className="upload-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="1.5" aria-hidden="true">
      <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v12m0-12l-4 4m4-4l4 4M4 16v2a1 1 0 001 1h14a1 1 0 001-1v-2" />
    </svg>
  );
}

/** Props for the {@link QueuePanel} component. */
interface QueuePanelProps {
  /** Current list of queued files. */
  files: FileItem[];
  /** Called when new file paths are added (from picker, drag-drop, or native drop). */
  onAddFiles: (paths: string[]) => void;
  /** Called to remove a single file from the queue by its id. */
  onRemoveFile: (id: string) => void;
  /** Called to clear the entire queue. */
  onClearFiles: () => void;
  /** i18n translation helper. */
  t: (key: string) => string;
}

export function QueuePanel({ files, onAddFiles, onRemoveFile, onClearFiles, t }: QueuePanelProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [showFormats, setShowFormats] = useState(false);
  const [formatsList, setFormatsList] = useState<string[]>([]);
  const dropzoneRef = useRef<HTMLDivElement>(null);

  // ── Effect: load supported formats from the API on mount ─────────────────
  // Falls back to a hardcoded list if the backend is unreachable.
  useEffect(() => {
    const loadFormats = async () => {
      try {
        const data = await api.getFormats();
        let list: string[] = [];
        if (Array.isArray(data)) {
          list = data.map((f: { extension: string }) => f.extension.replace(/^\./, '').toUpperCase());
        } else if (data && typeof data === 'object') {
          list = Object.keys(data).map(key => key.replace(/^\./, '').toUpperCase());
        }
        list.sort();
        setFormatsList(list);
      } catch {
        // Fallback to basic list if API fails
        setFormatsList([
          'AAC','AC3','AIFF','ALAC','AMR','APE','AU','CAF',
          'DFF','DSD','DSF','DTS','EAC3','FLAC','G722','G726',
          'GSM','ILBC','M4A','M4B','M4R','MKA','MP2','MP3',
          'OGA','OGG','OPUS','PCM','RAW','S16','S24','S32',
          'SPEEX','TAK','TTA','ULAW','WAV','WEBM','WMA','WV'
        ]);
      }
    };
    loadFormats();
  }, []);

  // ── Effect: poll for native drag-and-drop files every 500 ms ─────────────
  // The Rust backend stores files dropped onto the tao window in a global Vec.
  // This interval drains that queue and forwards paths to the parent component.
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const dropped = await api.getDroppedFiles();
        if (dropped && dropped.length > 0) {
          onAddFiles(dropped);
        }
      } catch { /* polling error, ignore */ }
    }, 500);
    return () => clearInterval(interval);
  }, [onAddFiles]);

  // ── Effect: attach native drag-and-drop handlers to the dropzone div ─────
  // Uses addEventListener instead of React synthetic events so that
  // dragover / dragleave / drop are handled on the exact DOM node captured
  // by dropzoneRef, enabling fine-grained `contains()` checks on leave.
  useEffect(() => {
    const handleDragOver = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(true);
    };

    const handleDragLeave = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (dropzoneRef.current && !dropzoneRef.current.contains(e.relatedTarget as Node)) {
        setIsDragging(false);
      }
    };

    const handleDrop = (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(false);
      
      const items = Array.from(e.dataTransfer?.files ?? []);
      if (items.length > 0) {
        const paths = items.map(item => (item as { path?: string }).path || item.name);
        onAddFiles(paths);
      }
    };

    const dropzone = dropzoneRef.current;
    if (dropzone) {
      dropzone.addEventListener('dragover', handleDragOver);
      dropzone.addEventListener('dragleave', handleDragLeave);
      dropzone.addEventListener('drop', handleDrop);
    }

    return () => {
      if (dropzone) {
        dropzone.removeEventListener('dragover', handleDragOver);
        dropzone.removeEventListener('dragleave', handleDragLeave);
        dropzone.removeEventListener('drop', handleDrop);
      }
    };
  }, [onAddFiles]);

  const handlePickFiles = usePickFiles(onAddFiles);

  /** Opens the native file picker when the dropzone is activated via keyboard (Enter / Space). */
  const handleDropzoneKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handlePickFiles();
    }
  };

  const isSingle = files.length === 1;
  const hasFiles = files.length > 0;

  return (
    <>
      <div className="card-header">
        <h2>{t('images.title')}</h2>
      </div>

      <div 
        id="drop-zone" 
        ref={dropzoneRef}
        className={`dropzone ${isDragging ? 'dragover' : ''}`}
        onClick={handlePickFiles}
        onKeyDown={handleDropzoneKeyDown}
        tabIndex={0}
        role="button"
        aria-label={t('a11y.dropzone')}
        style={{ cursor: 'pointer', flex: 1 }}
      >
        {!hasFiles && (
          <div className="dropzone-content" id="dropzone-prompt">
            <UploadIcon />
            <p className="prompt-text">
              {t('dropzone.prompt')}{' '}
              <span id="dropzone-add-files-link" 
                    className="toggle-formats-link"
                    onClick={(e) => { e.stopPropagation(); handlePickFiles(); }}>
                {t('dropzone.browse')}
              </span>
            </p>
            <FormatsToggle showFormats={showFormats} onToggle={() => setShowFormats(!showFormats)} t={t} formatsList={formatsList} />
          </div>
        )}

        {isSingle && (
          <div className="dropzone-preview" id="dropzone-preview">
            <img className="preview-thumb" src={`data:image/svg+xml;utf8,${encodeURIComponent(`<svg xmlns='http://www.w3.org/2000/svg' width='240' height='240' viewBox='0 0 240 240'>
  <defs>
    <linearGradient id='grad' x1='0%' y1='0%' x2='100%' y2='100%'>
      <stop offset='0%' stop-color='#150f2e' />
      <stop offset='100%' stop-color='#2b0754' />
    </linearGradient>
    <linearGradient id='border-grad' x1='0%' y1='0%' x2='100%' y2='100%'>
      <stop offset='0%' stop-color='#a855f7' stop-opacity='0.8' />
      <stop offset='100%' stop-color='#ec4899' stop-opacity='0.2' />
    </linearGradient>
  </defs>
  <rect x='10' y='10' width='220' height='220' rx='28' fill='url(#grad)' stroke='url(#border-grad)' stroke-width='2' />
  <path d='M85 70c0-4.4 3.6-8 8-8s8 3.6 8 8v60c0 4.4-3.6 8-8 8s-8-3.6-8-8V70z' fill='%23a855f7' fill-opacity='0.12' />
  <path d='M139 62c0-4.4 3.6-8 8-8s8 3.6 8 8v76c0 4.4-3.6 8-8 8s-8-3.6-8-8V62z' fill='%23a855f7' fill-opacity='0.12' />
  <rect x='89' y='85' width='6' height='30' rx='3' fill='%23c084fc' fill-opacity='0.5' />
  <rect x='101' y='75' width='6' height='50' rx='3' fill='%23c084fc' fill-opacity='0.6' />
  <rect x='113' y='90' width='6' height='20' rx='3' fill='%23c084fc' fill-opacity='0.5' />
  <rect x='125' y='78' width='6' height='44' rx='3' fill='%23c084fc' fill-opacity='0.7' />
  <rect x='137' y='82' width='6' height='36' rx='3' fill='%23c084fc' fill-opacity='0.6' />
  <rect x='149' y='88' width='6' height='24' rx='3' fill='%23c084fc' fill-opacity='0.5' />
  <rect x='50' y='155' width='140' height='50' rx='14' fill='%23a855f7' fill-opacity='0.2' stroke='%23c084fc' stroke-width='1.5' style='filter: drop-shadow(0px 0px 8px rgba(168, 85, 247, 0.4));' />
  <text x='120' y='186' font-family='"Outfit", "Inter", system-ui, -apple-system, sans-serif' font-size='20' font-weight='800' fill='%23f3f1f7' text-anchor='middle' letter-spacing='0.05em'>${(files[0]?.name.split('.').pop()?.toUpperCase() || 'AUDIO')}</text>
</svg>`)}`} alt="" aria-hidden="true" />
            <div className="preview-info" id="preview-info">
              <span className="preview-filename" id="preview-filename">{files[0]?.name || ''}</span>
              <span className="preview-meta" id="preview-meta">
                {files[0] ? `${t('queue.formatInfo')} ${files[0].name.split('.').pop()?.toUpperCase()} | ${t('queue.sizeInfo')} ${formatSize(files[0].size)}` : ''}
              </span>
            </div>
            <button type="button" className="btn-remove" id="btn-remove-image" onClick={(e) => { e.stopPropagation(); onClearFiles(); }}>
              {t('convert.remove')}
            </button>
          </div>
        )}

        {hasFiles && !isSingle && (
          <div className="dropzone-content" id="dropzone-prompt">
            <UploadIcon />
            <p className="prompt-text">
              <strong style={{ color: 'var(--primary)' }}>{files.length} {t('queue.filesCount')} — </strong>
              <span className="toggle-formats-link" style={{ fontSize: '0.85rem', cursor: 'pointer' }} onClick={(e) => { e.stopPropagation(); handlePickFiles(); }}>
                {t('queue.addMore')}
              </span>
            </p>
            <FormatsToggle showFormats={showFormats} onToggle={() => setShowFormats(!showFormats)} t={t} formatsList={formatsList} />
          </div>
        )}
      </div>

      {hasFiles && !isSingle && (
        <div id="batch-container" className="panel-batch">
          <div className="batch-header-row">
            <span className="batch-header-title">{t('batch.title')}</span>
            <button type="button" id="btn-clear-batch" style={{ fontSize: '0.75rem', color: 'var(--error)', background: 'none', border: 'none', cursor: 'pointer', padding: 0 }} onClick={onClearFiles}>
              {t('menu.file.clearList')}
            </button>
          </div>
          <div id="batch-list" className="batch-list" aria-live="polite">
            {files.map(file => {
              const ext = file.name.split('.').pop()?.toLowerCase() || '';
              const isProcessing = file.status === 'processing';
              const isCompleted = file.status === 'completed';
              const isFailed = file.status === 'failed';
              
              return (
                <div key={file.id} className="batch-item">
                  <div className="batch-item-left">
                    <div className="batch-item-badge" style={{ 
                      background: isProcessing 
                        ? 'linear-gradient(135deg, rgba(59,130,246,0.25), rgba(37,99,235,0.15))'
                        : isCompleted
                        ? 'linear-gradient(135deg, rgba(16,185,129,0.25), rgba(5,150,105,0.15))'
                        : isFailed
                        ? 'linear-gradient(135deg, rgba(239,68,68,0.25), rgba(220,38,38,0.15))'
                        : 'linear-gradient(135deg, rgba(168,85,247,0.2), rgba(236,72,153,0.2))',
                      borderColor: isProcessing
                        ? 'rgba(59,130,246,0.4)'
                        : isCompleted
                        ? 'rgba(16,185,129,0.4)'
                        : isFailed
                        ? 'rgba(239,68,68,0.4)'
                        : 'rgba(168,85,247,0.3)'
                    }}>
                      {ext.toUpperCase()}
                    </div>
                    <span className="batch-item-name" title={file.name}>{file.name}</span>
                  </div>
                    <button 
                      className="batch-item-remove" 
                      onClick={() => onRemoveFile(file.id)}
                      title={t('convert.remove')}
                    >
                      {t('convert.remove')}
                    </button>
                </div>
              );
            })}
          </div>
        </div>
      )}

    </>
  );
}
