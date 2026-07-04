/**
 * Trix Audio Converter � useQueue
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * Queue management hook for audio files
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useState, useCallback, useRef, useEffect } from 'react';
import { FileItem, ConvertOptions } from '@/types';
import { api } from '@/utils/api';

/** Aggregated result returned after a conversion batch finishes. */
interface ConversionResult {
  /** Number of files that converted successfully. */
  completed: number;
  /** Number of files that failed. */
  failed: number;
  /** Total files attempted. */
  total: number;
}

/** Status payload for a single file as returned by the backend `/api/status` endpoint. */
interface BackendFileStatus {
  /** Absolute path to the source file (primary match key). */
  input?: string;
  /** Filename without directory (fallback match key). */
  filename?: string;
  /** Backend status string: `"processing"`, `"completed"`, or `"error"`. */
  status?: string;
  /** Progress percentage 0–100. */
  progress?: number;
  /** Error message when status is `"error"`. */
  error?: string;
}

/**
 * Finds the backend status for a queue item by matching on `input` path first,
 * then falling back to filename comparison.
 */
function findFileStatus(fileEntries: BackendFileStatus[], file: FileItem): BackendFileStatus | undefined {
  return fileEntries.find(s => s.input === file.path || s.filename === file.name);
}

/**
 * Merges backend status updates into the local file list without discarding
 * any files that the backend has not yet reported on.
 */
function applyFileStatuses(files: FileItem[], fileEntries: BackendFileStatus[]): FileItem[] {
  return files.map(f => {
    const fileStatus = findFileStatus(fileEntries, f);
    if (!fileStatus) return f;
    return {
      ...f,
      status: mapBackendStatus(fileStatus.status, f.status),
      progress: fileStatus.progress || f.progress,
      error: fileStatus.error || f.error,
    };
  });
}

/**
 * Maps a raw backend status string to the TypeScript `FileItem['status']` union.
 * Returns `fallback` for any unknown or missing status to preserve the last known state.
 */
function mapBackendStatus(backend: string | undefined, fallback: FileItem['status']): FileItem['status'] {
  if (backend === 'completed') return 'completed';
  if (backend === 'error') return 'failed';
  if (backend === 'processing') return 'processing';
  return fallback;
}

/**
 * Core queue management hook for audio/video file conversion.
 *
 * Responsibilities:
 * - Maintains the ordered list of queued files.
 * - Starts and polls a conversion job via the Axum backend API.
 * - Falls back to a simulated progress bar when the backend omits a `jobId`.
 * - Cleans up polling intervals on unmount.
 */
export function useQueue() {
  const [files, setFiles] = useState<FileItem[]>([]);
  const [isConverting, setIsConverting] = useState(false);
  const [progress, setProgress] = useState(0);
  const [conversionResult, setConversionResult] = useState<ConversionResult | null>(null);
  /** Ref for the `/api/status` polling interval so it can be cancelled at any time. */
  const statusIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  /** Ref for the simulated-progress fallback interval (used when no `jobId` is returned). */
  const fallbackIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  /** `true` while the fallback progress animation is running. */
  const fallbackRef = useRef(false);
  /** How many files were included in the fallback batch (used to build the final result). */
  const fallbackCountRef = useRef(0);

  // When the fallback progress animation reaches 100%, finalize the conversion
  // and build a synthetic result (all files assumed successful).
  useEffect(() => {
    if (fallbackRef.current && isConverting && progress >= 100) {
      fallbackRef.current = false;
      setIsConverting(false);
      setFiles(prev => prev.map(f => f.status === 'processing' ? { ...f, status: 'completed' as const, progress: 100 } : f));
      setConversionResult({ completed: fallbackCountRef.current, failed: 0, total: fallbackCountRef.current });
    }
  }, [isConverting, progress]);

  /**
   * Scans the given file/folder paths via the backend and appends any new,
   * non-duplicate files to the queue. Already-queued paths are skipped.
   */
  const addFiles = useCallback(async (paths: string[]) => {
    try {
      const newFiles = await api.scanFolders(paths);
      setFiles(prev => {
        const existingPaths = new Set(prev.map(f => f.path));
        const filteredNewFiles = newFiles.filter(f => !existingPaths.has(f.path));
        return [...prev, ...filteredNewFiles];
      });
    } catch (error) {
      console.error('Failed to add files:', error);
    }
  }, []);

  /** Removes the file with the given `id` from the queue. */
  const removeFile = useCallback((id: string) => {
    setFiles(prev => prev.filter(f => f.id !== id));
  }, []);

  /** Empties the entire queue. */
  const clearFiles = useCallback(() => {
    setFiles([]);
  }, []);

  /**
   * Starts a 500 ms polling loop against `/api/status/:jobId`.
   * Updates individual file statuses and overall progress on each tick.
   * Stops automatically when `status.converting` becomes `false`.
   */
  const pollStatus = useCallback((jobId: string) => {
    if (statusIntervalRef.current) {
      clearInterval(statusIntervalRef.current);
    }

    statusIntervalRef.current = setInterval(async () => {
      try {
        const status = await api.getConversionStatus(jobId);
        if (status && status.files) {
          const fileEntries = Object.values(status.files) as BackendFileStatus[];
          setFiles(prev => applyFileStatuses(prev, fileEntries));
        }

        const overallProgress = status?.progress || 0;
        setProgress(overallProgress);

        // Stop polling as soon as the backend says it's done (success OR error)
        if (status?.converting === false) {
          if (statusIntervalRef.current) {
            clearInterval(statusIntervalRef.current);
            statusIntervalRef.current = null;
          }
          setIsConverting(false);

          // Update file statuses from the final backend state
          const fileEntries = Object.values(status.files || {}) as BackendFileStatus[];
          setFiles(prev => {
            const mapped = applyFileStatuses(prev, fileEntries);
            return mapped.map(f => {
              if (f.status === 'processing') return { ...f, status: 'completed' as const, progress: 100 };
              return f;
            });
          });

          // Count results directly from backend status (not from async setFiles)
          let completedCount = 0;
          let failedCount = 0;
          for (const entry of fileEntries) {
            if (entry.status === 'completed') completedCount++;
            else if (entry.status === 'error') failedCount++;
            else failedCount++;
          }
          if (completedCount === 0 && failedCount === 0) completedCount = fileEntries.length;

          setConversionResult({ completed: completedCount, failed: failedCount, total: completedCount + failedCount });
          if (overallProgress >= 100 || status.converted > 0) setProgress(100);
        }
      } catch {
        // Silently continue polling
      }
    }, 500);
  }, []);

  // Cleanup: cancel both polling intervals on component unmount to prevent
  // state updates on an unmounted component.
  useEffect(() => {
    return () => {
      if (statusIntervalRef.current) {
        clearInterval(statusIntervalRef.current);
      }
      if (fallbackIntervalRef.current) {
        clearInterval(fallbackIntervalRef.current);
      }
    };
  }, []);

  /**
   * Starts a conversion job for all files with status `pending`, `failed`, or `completed`.
   *
   * Flow:
   * 1. Marks files as `processing`.
   * 2. Calls `/api/start` on the backend.
   * 3. If a `jobId` is returned, starts the real-time polling via {@link pollStatus}.
   * 4. Otherwise, starts the fallback simulated-progress timer.
   */
  const startConversion = useCallback(async (options: Partial<ConvertOptions> & { outputDirectory?: string; outputInSameFolder?: boolean; outputSuffix?: string }) => {
    const pendingFiles = files.filter(f => f.status === 'pending' || f.status === 'failed' || f.status === 'completed');
    if (pendingFiles.length === 0) return;

    setFiles(prev => prev.map(f =>
      (f.status === 'pending' || f.status === 'failed' || f.status === 'completed')
        ? { ...f, status: 'processing' as const, progress: 0, error: undefined }
        : f
    ));

    setIsConverting(true);
    setProgress(0);
    
    try {
      const result = await api.startConversion({
        ...options,
        files: pendingFiles.map(f => f.path),
      }) as { jobId?: string; success?: boolean; error?: string };

      // Check if backend reported an error even with 200 OK
      if (result && result.success === false) {
        const errMsg = result.error || 'Falha na conversão';
        console.error('Backend conversion error:', errMsg);
        setIsConverting(false);
        setFiles(prev => prev.map(f => f.status === 'processing' ? { ...f, status: 'failed', error: errMsg } : f));
        return;
      }

      if (result?.jobId) {
        pollStatus(result.jobId);
      } else {
        fallbackRef.current = true;
        fallbackCountRef.current = pendingFiles.length;
        fallbackIntervalRef.current = setInterval(() => {
          setProgress(prev => {
            if (prev >= 100) {
              if (fallbackIntervalRef.current) {
                clearInterval(fallbackIntervalRef.current);
                fallbackIntervalRef.current = null;
              }
              return 100;
            }
            const increment = Math.max(1, Math.floor(100 / pendingFiles.length));
            return Math.min(prev + increment, 100);
          });
        }, 500);
      }
    } catch (error) {
      console.error('Conversion failed:', error);
      setIsConverting(false);
      setFiles(prev => prev.map(f => f.status === 'processing' ? { ...f, status: 'failed', error: 'Falha na conversão' } : f));
    }
  }, [files, pollStatus]);

  /** Clears the last conversion result object (used after the result modal is dismissed). */
  const clearConversionResult = useCallback(() => {
    setConversionResult(null);
  }, []);

  return {
    files,
    addFiles,
    removeFile,
    clearFiles,
    startConversion,
    isConverting,
    progress,
    conversionResult,
    clearConversionResult,
  };
}
