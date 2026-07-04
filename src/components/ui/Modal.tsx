/**
 * Trix Audio Converter � Modal
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * Reusable modal dialog component
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useEffect, useRef } from 'react';
import { ModalState } from '@/types';

/** Props for the {@link Modal} component. */
interface ModalProps {
  /** Full modal state object including type, title, message, and optional callbacks. */
  modal: ModalState;
  /** Called when the modal should be dismissed (Escape key, backdrop click, or OK button). */
  onClose: () => void;
  /** i18n translation helper (used for button labels). */
  t: (key: string) => string;
}

export function Modal({ modal, onClose, t }: ModalProps) {
  const modalRef = useRef<HTMLDivElement>(null);
  const previousActiveElement = useRef<HTMLElement | null>(null);
  const firstFocusableRef = useRef<HTMLElement | null>(null);
  const lastFocusableRef = useRef<HTMLElement | null>(null);

  /**
   * SVG icon set keyed by modal type.
   * Each icon uses the relevant semantic colour from the CSS design tokens:
   * `--primary` for info/prompt, amber for confirm, `--error` for error, green for success.
   */
  const iconMap = {
    info: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--primary)" strokeWidth="2">
        <circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/>
      </svg>
    ),
    confirm: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" strokeWidth="2">
        <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
      </svg>
    ),
    error: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--error)" strokeWidth="2">
        <circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/>
      </svg>
    ),
    success: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#10b981" strokeWidth="2">
        <path d="M22 11.08V12a10 10 0 11-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/>
      </svg>
    ),
    prompt: (
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--primary)" strokeWidth="2">
        <path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" />
        <path d="M18.5 2.5a2.121 2.121 0 113 3L12 15l-4 1 1-4 9.5-9.5z" />
      </svg>
    ),
  };

  // Focus trap + restore focus on unmount
  // Focus trap: on mount, moves focus to the first focusable element inside the modal
  // and cycles Tab / Shift+Tab within the dialog boundary.
  // On unmount, restores focus to whichever element triggered the modal open.
  useEffect(() => {
    previousActiveElement.current = document.activeElement as HTMLElement;
    const modalEl = modalRef.current;
    if (!modalEl) return;

    // Find first and last focusable elements
    const focusableSelector = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';
    const focusableElements = modalEl.querySelectorAll<HTMLElement>(focusableSelector);
    if (focusableElements.length > 0) {
      firstFocusableRef.current = focusableElements[0];
      lastFocusableRef.current = focusableElements[focusableElements.length - 1];
      firstFocusableRef.current.focus();
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
        return;
      }

      if (e.key === 'Tab') {
        const focusable = Array.from(modalEl.querySelectorAll<HTMLElement>('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])')).filter(el => !el.hasAttribute('disabled') && el.offsetParent !== null);
        if (focusable.length === 0) return;

        const first = focusable[0];
        const last = focusable[focusable.length - 1];

        if (e.shiftKey) {
          if (document.activeElement === first) {
            e.preventDefault();
            last.focus();
          }
        } else {
          if (document.activeElement === last) {
            e.preventDefault();
            first.focus();
          }
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      // Restore focus to element that opened the modal
      if (previousActiveElement.current && previousActiveElement.current.focus) {
        previousActiveElement.current.focus();
      }
    };
  }, [onClose]);

  if (!modal.open) return null;

  return (
    <div className="modal-overlay" onClick={onClose} role="presentation">
      <div
        ref={modalRef}
        className={`modal-content${modal.isScrollable ? ' modal-scrollable' : ''}${modal.hideIcon ? ' modal-no-icon' : ''}`}
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="modal-title"
        aria-describedby="modal-message"
      >
        {modal.isScrollable && (
          <div className="modal-logo">
            <img src="/trix_logo_sunset.png" alt="Trix" style={{ width: 64, height: 64, borderRadius: 16 }} />
          </div>
        )}
        {!modal.isScrollable && !modal.hideIcon && (
          <div className="modal-icon" aria-hidden="true">
            {iconMap[modal.type]}
          </div>
        )}
        <h3 id="modal-title" className="modal-title">{modal.title}</h3>
        {modal.isScrollable ? (
          <div className="modal-scroll-body">
            <p id="modal-message" className="modal-message">{modal.message}</p>
          </div>
        ) : (
          <p id="modal-message" className="modal-message">{modal.message}</p>
        )}
        {modal.type === 'prompt' && (
          <div style={{ marginTop: '16px', marginBottom: '16px', width: '100%' }}>
            <input
              type="text"
              id="modal-prompt-input"
              className="text-input"
              style={{
                width: '100%',
                padding: '12px 16px',
                borderRadius: '8px',
                border: '1px solid var(--border-color)',
                background: 'rgba(255, 255, 255, 0.05)',
                color: 'var(--text-primary)',
                fontSize: '0.9rem',
                outline: 'none',
                transition: 'border-color 0.2s',
              }}
              placeholder={modal.promptPlaceholder}
              defaultValue={modal.promptDefaultValue}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  const input = document.getElementById('modal-prompt-input') as HTMLInputElement;
                  if (input) {
                    modal.onPromptConfirm?.(input.value);
                    onClose();
                  }
                }
              }}
              autoFocus
            />
          </div>
        )}
        <div className="modal-actions">
          {modal.type === 'prompt' ? (
            <>
              <button
                className="modal-btn modal-btn-cancel"
                onClick={onClose}
                aria-label={t('modal.cancel')}
              >
                {t('modal.cancel')}
              </button>
              <button
                className="modal-btn modal-btn-confirm"
                onClick={() => {
                  const input = document.getElementById('modal-prompt-input') as HTMLInputElement;
                  if (input) {
                    modal.onPromptConfirm?.(input.value);
                    onClose();
                  }
                }}
                aria-label={t('modal.confirm')}
              >
                {t('modal.confirm')}
              </button>
            </>
          ) : modal.type === 'confirm' ? (
            <>
              <button
                className="modal-btn modal-btn-cancel"
                onClick={onClose}
                aria-label={t('modal.cancel')}
              >
                {t('modal.cancel')}
              </button>
              <button
                className="modal-btn modal-btn-confirm"
                onClick={async () => { try { await modal.onConfirm?.(); } catch { /* onConfirm error */ } }}
                aria-label={t('modal.confirm')}
              >
                {t('modal.confirm')}
              </button>
            </>
          ) : (
            <button
              className={`modal-btn modal-btn-ok${modal.hideIcon ? ' modal-btn-full' : ''}`}
              onClick={onClose}
              aria-label={t('modal.understood') || 'OK'}
            >
              {modal.buttonLabel || 'OK'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}