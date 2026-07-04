/**
 * Trix Audio Converter � usePersistedState
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * State persistence hook using localStorage
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useState, useEffect } from 'react';

/**
 * Generic state hook that persists its value to `localStorage` under `key`.
 *
 * - On first render the lazy initializer reads the stored JSON; falls back to `defaultValue`
 *   if the key is missing or if the stored value cannot be parsed.
 * - After every state change the new value is serialised and written back to `localStorage`.
 *
 * @param key          - The `localStorage` key to read from and write to.
 * @param defaultValue - The initial state value used when no persisted value is found.
 * @returns A `[value, setValue]` tuple identical to `useState`.
 */
export function usePersistedState<T>(key: string, defaultValue: T): [T, React.Dispatch<React.SetStateAction<T>>] {
  const [state, setState] = useState<T>(() => {
    try {
      const saved = localStorage.getItem(key);
      if (saved) {
        return JSON.parse(saved) as T;
      }
    } catch {
      // corrupted localStorage data — reset to defaults
    }
    return defaultValue;
  });

  useEffect(() => {
    try {
      localStorage.setItem(key, JSON.stringify(state));
    } catch {
      // quota exceeded or storage unavailable
    }
  }, [key, state]);

  return [state, setState];
}
