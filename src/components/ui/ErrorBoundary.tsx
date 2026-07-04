/**
 * Trix Audio Converter � ErrorBoundary
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * React error boundary for crash handling
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { Component, ReactNode } from 'react';

/** Props for the {@link ErrorBoundary} component. */
interface Props {
  /** The component subtree to protect. */
  children: ReactNode;
  /** i18n translation helper (used to render the error recovery UI). */
  t: (key: string) => string;
}

/** Internal state for {@link ErrorBoundary}. */
interface State {
  /** Whether a render error has been caught. */
  hasError: boolean;
  /** The caught `Error` object, or `null` when no error is active. */
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, error: null };

  /**
   * React lifecycle: called when a descendant throws during rendering.
   * Returns the new state that triggers the fallback UI.
   */
  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  /**
   * Renders the fallback recovery screen when `hasError` is `true`,
   * otherwise renders children normally.
   */
  render() {
    if (this.state.hasError) {
      const { t } = this.props;
      return (
        <div style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100vh',
          padding: '2rem',
          fontFamily: 'system-ui, sans-serif',
          background: '#0a0515',
          color: '#f3f1f7',
        }}>
          <h2 style={{ color: '#ef4444', marginBottom: '1rem' }}>{t('errorBoundary.title')}</h2>
          <p style={{ opacity: 0.7, marginBottom: '1.5rem', textAlign: 'center' }}>
            {t('errorBoundary.message')}
          </p>
          <button
            onClick={() => {
              this.setState({ hasError: false, error: null });
              window.location.reload();
            }}
            style={{
              padding: '0.75rem 2rem',
              background: '#a855f7',
              color: '#fff',
              border: 'none',
              borderRadius: '8px',
              cursor: 'pointer',
              fontSize: '1rem',
            }}
          >
            {t('errorBoundary.reload')}
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
