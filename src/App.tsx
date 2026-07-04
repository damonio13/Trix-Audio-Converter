/**
 * Trix Audio Converter � App
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * Main application component
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useState, useCallback, useRef, useEffect } from 'react'
import { TitleBar } from '@/components/layout/TitleBar'
import { QueuePanel } from '@/components/panels/QueuePanel'
import { FormatSelector } from '@/components/panels/FormatSelector'
import { SettingsPanel } from '@/components/panels/SettingsPanel'
import { Modal } from '@/components/ui/Modal'
import { ErrorBoundary } from '@/components/ui/ErrorBoundary'
import { useTheme } from '@/hooks/useTheme'
import { useI18n } from '@/hooks/useI18n'
import { useQueue } from '@/hooks/useQueue'
import { useSettings } from '@/hooks/useSettings'
import { usePickFiles } from '@/hooks/usePickFiles'
import { useOnline } from '@/hooks/useOnline'
import { ModalState } from '@/types'
import { api } from '@/utils/api'

export default function App() {
  // ── Cross-cutting hooks ──────────────────────────────────────────
  const { theme, setTheme } = useTheme()
  const { language, setLanguage, t } = useI18n()
  const { files, addFiles, removeFile, clearFiles, startConversion, isConverting, progress, conversionResult, clearConversionResult } = useQueue()
  const { settings, updateSettings, resetSettings } = useSettings()
  /** Imperative modal state — driven by `showModal()` helper below. */
  const [modal, setModal] = useState<ModalState>({ open: false, type: 'info', title: '', message: '' })
  /** Ref used to detect the *falling edge* of `isConverting` (i.e. conversion just finished). */
  const wasConvertingRef = useRef(false)
  const isOnline = useOnline()

  // ── Effect: show completion modal when conversion finishes ──────────────────
  // We track the *previous* value of `isConverting` in a ref to detect the
  // transition from `true` → `false` without adding it to the dependency array
  // (which would cause the effect to run on every intermediate render).
  useEffect(() => {
    if (wasConvertingRef.current && !isConverting && conversionResult) {
      const suffix = settings.outputSuffix || '_trix'
      const title = conversionResult.failed === 0 ? t('modal.convertSuccess') : t('modal.convertPartial')
      let message = ''
      if (conversionResult.completed > 0) {
        message += `${conversionResult.completed} ${t('convert.resultSuccess')}`
      }
      if (conversionResult.failed > 0) {
        message += `${conversionResult.failed > 0 && conversionResult.completed > 0 ? '\n' : ''}${conversionResult.failed} ${t('convert.resultFailed')}`
      }
      const location = settings.outputInSameFolder
        ? `${t('convert.resultLocationSame')} '${suffix}'.`
        : `${t('convert.resultLocationDest')} '${suffix}'.`
      message += `\n\n${location}`
      setModal({ open: true, type: 'success', title, message, buttonLabel: t('modal.understood') || 'Entendido', hideIcon: true })
      clearConversionResult()
    }
    wasConvertingRef.current = isConverting
  }, [isConverting, conversionResult, settings.outputSuffix, settings.outputInSameFolder, t, clearConversionResult])

  const handlePickFiles = usePickFiles(addFiles)

  /** Opens a modal imperatively; defaults to `'info'` type. */
  const showModal = useCallback((title: string, message: string, type: ModalState['type'] = 'info') => {
    setModal({ open: true, type, title, message })
  }, [])

  return (
    <ErrorBoundary t={t}>
      <TitleBar
        theme={theme}
        onToggleTheme={(id) => setTheme(id)}
        language={language}
        onSetLanguage={setLanguage}
        onAddFiles={handlePickFiles}
        onClearQueue={clearFiles}
        onOpenDestination={() => { api.openFolder(settings.outputDirectory || '.').catch(() => {}); }}
        onResetSettings={() => { resetSettings(); showModal(t('settings.title'), t('modal.resetSuccess'), 'success') }}
        onClearCache={() => setModal({ open: true, type: 'confirm', title: t('menu.tools.clearCache'), message: t('modal.clearCacheConfirm'), onConfirm: async () => {
          try { await api.clearCache(true); showModal(t('menu.tools.clearCache'), t('modal.cacheCleared'), 'success') }
          catch { showModal(t('video.error'), t('modal.cacheClearFailed'), 'error') }
        }})}
        onShowFormats={() => setModal({ open: true, type: 'info', title: t('modal.formatsTitle'), message: t('modal.formatsList'), isScrollable: true })}
        onShowGuide={() => setModal({ open: true, type: 'info', title: t('modal.guideTitle'), message: t('modal.guideContent'), isScrollable: true })}
        onRegisterContextMenu={() => {
          api.registerContextMenu()
            .then((res) => {
              if (res && res.success) {
                showModal(t('menu.tools.registerCtx'), t('ctxMenu.registered'), 'success');
              } else {
                showModal(t('video.error'), res?.error || t('ctxMenu.registerFailed'), 'error');
              }
            })
            .catch((err) => showModal(t('video.error'), err.message || t('ctxMenu.registerFailed'), 'error'))
        }}
        onUnregisterContextMenu={() => {
          api.unregisterContextMenu()
            .then((res) => {
              if (res && res.success) {
                showModal(t('menu.tools.unregisterCtx'), t('ctxMenu.unregistered'), 'success');
              } else {
                showModal(t('video.error'), res?.error || t('ctxMenu.unregisterFailed'), 'error');
              }
            })
            .catch((err) => showModal(t('video.error'), err.message || t('ctxMenu.unregisterFailed'), 'error'))
        }}
        isOnline={isOnline}
        t={t}
        settings={settings}
        onSettingsChange={updateSettings}
        onShowPromptModal={(opts) => {
          setModal({
            open: true,
            type: 'prompt',
            title: opts.title,
            message: opts.message,
            promptPlaceholder: opts.placeholder,
            promptDefaultValue: opts.defaultValue,
            onPromptConfirm: opts.onConfirm
          });
        }}
      />

      <div className="background-gradients">
        <div className="blob blob-purple" />
        <div className="blob blob-blue" />
        <div className="blob blob-magenta" />
      </div>

      <Modal modal={modal} onClose={() => setModal(prev => ({ ...prev, open: false }))} t={t} />

      <main className="app-container">
        <header className="app-header">
          <h1 className="logo-title">Trix <span>{t('app.subtitle')}</span></h1>
        </header>

        <section className="dashboard-layout" aria-live="polite">
          <div className="card panel-left">
            <QueuePanel
              files={files}
              onAddFiles={addFiles}
              onRemoveFile={removeFile}
              onClearFiles={clearFiles}
              t={t}
            />
          </div>

          <div className="card panel-right">
              <div className="card-header">
                <h2>{t('settings.title') || 'Configurações'}</h2>
              </div>
              <div className="panel-right-inner">
                <div className="settings-grid">
                  <div className="settings-left-col">
                    <FormatSelector
                      formats={settings.defaultFormats}
                      onFormatsChange={(formats) => updateSettings({ defaultFormats: formats, defaultFormat: formats[0] || '' })}
                      t={t}
                    />
                  </div>
                  <div className="settings-right-col">
                    <SettingsPanel
                      settings={settings}
                      onSettingsChange={updateSettings}
                      t={t}
                      files={files}
                      showModal={showModal}
                    />
                  </div>
                </div>
                <div className="form-footer">
                  {isConverting ? (
                    <div className="progress-container" style={{ flex: 1 }} role="progressbar" aria-valuenow={progress} aria-valuemin={0} aria-valuemax={100} aria-label={t('a11y.progress')}>
                      <div className="progress-bar-bg">
                        <div className="progress-bar-fill" style={{ width: `${progress}%` }} />
                      </div>
                      <div className="progress-text">
                        <span>{t('converting') || 'Convertendo...'}</span>
                        <span>{progress}%</span>
                      </div>
                    </div>
                  ) : (
                    <button
                      type="submit"
                      className="btn-submit"
                      id="btn-convert"
                      title={
                        files.length === 0
                          ? (t('convert.noFiles') || 'Adicione arquivos para converter')
                          : !(settings.defaultFormats.length > 0 || settings.defaultFormat)
                            ? (t('convert.selectFormat') || 'Selecione um formato de saída primeiro')
                            : ''
                      }
                      onClick={() => {
                        const formats = settings.defaultFormats.length > 0 ? settings.defaultFormats : [settings.defaultFormat].filter(Boolean);
                        if (formats.length === 0) {
                          showModal(t('convert.selectFormat') || 'Formato necessário', t('convert.selectFormat') || 'Selecione um formato de saída primeiro.', 'info');
                          return;
                        }
                        const options = {
                          formats,
                          format: formats[0],
                          sampleRate: settings.defaultSampleRate,
                          channels: settings.defaultChannels,
                          bitRate: settings.defaultBitrate ? settings.defaultBitrate + 'k' : undefined,
                          volume: settings.volume,
                          codecCopy: settings.codecCopy,
                          trimStart: settings.trimStart || undefined,
                          trimEnd: settings.trimEnd || undefined,
                          outputDirectory: settings.outputDirectory,
                          outputInSameFolder: settings.outputInSameFolder,
                          outputSuffix: settings.outputSuffix,
                        };
                        startConversion(options);
                      }}
                      disabled={isConverting || files.length === 0 || (settings.defaultFormats.length === 0 && !settings.defaultFormat)}
                    >
                      {t('convert.button') || 'Converter Áudio'}
                    </button>
                  )}
                </div>
              </div>
            </div>
          </section>
      </main>
    </ErrorBoundary>
  )
}
