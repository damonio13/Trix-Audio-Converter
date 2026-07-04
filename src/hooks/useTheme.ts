/**
 * Trix Audio Converter � useTheme
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * Theme management hook
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useEffect } from 'react';
import { usePersistedState } from './usePersistedState';

/** Union of all valid theme identifiers used as `localStorage` keys and CSS class names. */
export type ThemeId = 'original' | 'aurora' | 'cyberpunk' | 'sunset' | 'emerald';

/** A registered UI theme with its id and the CSS body class to apply. */
interface Theme {
  /** Matches a `ThemeId` value and is used as the persistence key. */
  id: ThemeId;
  /** CSS class added to `<body>` when this theme is active. Empty string for the default theme. */
  class: string;
}

/** All 5 available themes in display order. `"original"` has an empty class (default CSS variables). */
export const AVAILABLE_THEMES: Theme[] = [
  { id: 'original',  class: '' },
  { id: 'aurora',    class: 'theme-aurora' },
  { id: 'cyberpunk', class: 'theme-cyberpunk' },
  { id: 'sunset',    class: 'theme-sunset' },
  { id: 'emerald',   class: 'theme-emerald' },
];

export function useTheme() {
  const [theme, setTheme] = usePersistedState<ThemeId>('trix_theme', 'sunset');

/**
 * Applies the active theme to `<body>` by toggling the appropriate CSS class
 * whenever `theme` changes. Removes all other theme classes first to prevent
 * stacking.
 */
  useEffect(() => {
    const body = document.body;
    body.classList.remove('theme-aurora', 'theme-cyberpunk', 'theme-sunset', 'theme-emerald');
    const t = AVAILABLE_THEMES.find(t => t.id === theme);
    if (t && t.class) body.classList.add(t.class);
  }, [theme]);

  return { theme, setTheme };
}
