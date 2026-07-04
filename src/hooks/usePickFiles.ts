/**
 * Trix Audio Converter � usePickFiles
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * Native file picker hook
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useCallback } from 'react';
import { api } from '@/utils/api';

/**
 * Returns a stable callback that opens the native OS file-picker dialog
 * (implemented by the Rust backend via `api.pickFiles`) and forwards
 * the selected file paths to `onAddFiles`.
 *
 * @param onAddFiles - Callback invoked with the array of selected file paths.
 */
export function usePickFiles(onAddFiles: (paths: string[]) => void) {
  return useCallback(async () => {
    try {
      const paths = await api.pickFiles();
      if (paths && paths.length > 0) {
        onAddFiles(paths);
      }
    } catch (error) {
      console.error('Failed to pick files:', error);
    }
  }, [onAddFiles]);
}
