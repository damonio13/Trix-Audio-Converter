/**
 * Trix Audio Converter � FormatSelector
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * Output format selection grid
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { useState, useEffect, useMemo } from 'react';
import { AudioFormat } from '@/types';
import { api } from '@/utils/api';

/** Props for the {@link FormatSelector} component. */
interface FormatSelectorProps {
  /** Currently selected format extensions (e.g. `["mp3", "flac"]`). */
  formats: string[];
  /** Called whenever the user toggles a format pill. */
  onFormatsChange: (formats: string[]) => void;
  /** i18n translation helper. */
  t: (key: string) => string;
}

/** The 10 most commonly used audio formats, highlighted with a star in the grid. */
const POPULAR_FORMATS = new Set(['mp3', 'wav', 'flac', 'aac', 'ogg', 'opus', 'wma', 'm4a', 'aiff', 'alac']);

/**
 * Fallback format list used when the backend API is unreachable.
 * Contains the most common 21 formats so the UI remains functional offline.
 */
const FALLBACK_FORMATS: AudioFormat[] = [
  { id: '.mp3', name: 'MP3', extension: '.mp3' },
  { id: '.wav', name: 'WAV', extension: '.wav' },
  { id: '.flac', name: 'FLAC', extension: '.flac' },
  { id: '.aac', name: 'AAC', extension: '.aac' },
  { id: '.ogg', name: 'OGG', extension: '.ogg' },
  { id: '.opus', name: 'Opus', extension: '.opus' },
  { id: '.wma', name: 'WMA', extension: '.wma' },
  { id: '.m4a', name: 'M4A', extension: '.m4a' },
  { id: '.aiff', name: 'AIFF', extension: '.aiff' },
  { id: '.alac', name: 'ALAC', extension: '.alac' },
  { id: '.wv', name: 'WavPack', extension: '.wv' },
  { id: '.ape', name: 'APE', extension: '.ape' },
  { id: '.dsf', name: 'DSF', extension: '.dsf' },
  { id: '.dff', name: 'DFF', extension: '.dff' },
  { id: '.mp2', name: 'MP2', extension: '.mp2' },
  { id: '.m4b', name: 'M4B', extension: '.m4b' },
  { id: '.m4r', name: 'M4R', extension: '.m4r' },
  { id: '.amr', name: 'AMR', extension: '.amr' },
  { id: '.dts', name: 'DTS', extension: '.dts' },
  { id: '.ac3', name: 'AC3', extension: '.ac3' },
  { id: '.eac3', name: 'E-AC3', extension: '.eac3' },
];

/**
 * Maps category names to the keywords that appear in a format's description.
 * Used to make the search box recognise queries like "lossless" or "podcast".
 */
const CATEGORY_KEYWORDS: Record<string, string[]> = {
  'lossless':    ['lossless', 'sem perda', 'cd', 'qualidade', 'archiv', 'preserv'],
  'lossy':       ['lossy', 'com perda', 'compact', 'streaming', 'web', 'podcast', 'rádio', 'radio'],
  'uncompressed':['uncompressed', 'raw', 'pcm', 'bruto', 'edit'],
  'professional':['studio', 'profissional', 'mastering', 'post-produção'],
  'retro':       ['retro', 'antigo', '8-bit', 'game', 'jogo', 'atari', 'amiga', 'commodore'],
  'broadcast':   ['broadcast', 'tv', 'rádio', 'dab', 'digital'],
  'mobile':      ['mobile', 'celular', 'phone', 'android', 'ios'],
  'compressed':  ['comprimido', 'small', 'pequeno', 'compact'],
};

/**
 * Maps common use-case words to the format extensions they imply.
 * Allows the search box to accept queries like "podcast" or "ringtone"
 * and show the matching formats even though no format is literally named that.
 */
const SEARCH_ALIASES: Record<string, string[]> = {
  'podcast':     ['mp3', 'aac', 'opus', 'ogg'],
  'música':      ['mp3', 'flac', 'wav', 'aac', 'ogg', 'opus', 'm4a'],
  'music':       ['mp3', 'flac', 'wav', 'aac', 'ogg', 'opus', 'm4a'],
  'qualidade':   ['flac', 'wav', 'aiff', 'alac', 'ape'],
  'quality':     ['flac', 'wav', 'aiff', 'alac', 'ape'],
  'web':         ['mp3', 'aac', 'ogg', 'opus', 'webm'],
  'ringtone':    ['mp3', 'm4r', 'm4a', 'ogg'],
  'editing':     ['wav', 'aiff', 'flac'],
  'livre':       ['flac', 'wav', 'aiff', 'alac', 'ape', 'wv'],
  'free':        ['flac', 'wav', 'aiff', 'alac', 'ape', 'wv'],
  'compatível':  ['mp3', 'wav', 'aac', 'wma'],
  'compatible':  ['mp3', 'wav', 'aac', 'wma'],
};

export function FormatSelector({ formats, onFormatsChange, t }: FormatSelectorProps) {
  const [allFormats, setAllFormats] = useState<AudioFormat[]>([]);
  const [search, setSearch] = useState('');
  const [showAll, setShowAll] = useState(true);

  useEffect(() => {
    const loadFormats = async () => {
      try {
        const data = await api.getFormats();
        let arr: AudioFormat[];
        if (Array.isArray(data)) {
          arr = data;
        } else if (data && typeof data === 'object') {
          arr = Object.entries(data).map(([key, val]) => ({
            id: key,
            name: key,
            extension: val.ext || '',
          }));
        } else {
          arr = [];
        }
        setAllFormats(arr);
      } catch (error) {
        console.error('Failed to load formats, using fallback:', error);
        setAllFormats(FALLBACK_FORMATS);
      }
    };
    loadFormats();
  }, []);

  // De-duplicate by extension (the API can return the same extension under
  // multiple keys for different codec variants — e.g. ".mp3" → libmp3lame vs fraunhofer).
  const uniqueFormats = useMemo(() => {
    const seen = new Set<string>();
    return allFormats.filter(f => {
      const key = f.extension.toLowerCase();
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    });
  }, [allFormats]);

  // Apply the "Popular only" toggle, then the search query (alias → keyword → substring).
  const filteredFormats = useMemo(() => {
    let result = uniqueFormats;

    if (!showAll) {
      result = result.filter(f => POPULAR_FORMATS.has(f.extension.toLowerCase().replace(/^\./, '')));
    }

    if (!search.trim()) return result;

    const q = search.toLowerCase().trim();

    const aliasExtensions = SEARCH_ALIASES[q];
    if (aliasExtensions) {
      return result.filter(f => aliasExtensions.includes(f.extension.toLowerCase().replace(/^\./, '')));
    }

    return result.filter(f => {
      const ext = f.extension.toLowerCase().replace(/^\./, '');
      if (ext.includes(q)) return true;
      if (f.name.toLowerCase().includes(q)) return true;
      if (f.id.toLowerCase().includes(q)) return true;

      for (const [, keywords] of Object.entries(CATEGORY_KEYWORDS)) {
        if (keywords.some(k => q.includes(k) || k.includes(q))) {
          return true;
        }
      }
      return false;
    });
  }, [uniqueFormats, search, showAll]);

  /** Toggles `formatId` in the selected formats list. Normalises the id to lowercase without a leading dot. */
  const toggleFormat = (formatId: string) => {
    const normalized = formatId.toLowerCase().replace(/^\./, '');
    if (formats.includes(normalized)) {
      onFormatsChange(formats.filter(f => f !== normalized));
    } else {
      onFormatsChange([...formats, normalized]);
    }
  };

  return (
    <div className="control-group" style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div className="label-header">
        <label className="group-label">{t('output.title') || 'Formato de Saída'}</label>
        <input
          type="text"
          id="search-format-input"
          className="search-format-input"
          placeholder={t('output.search') || '🔍 Buscar formato...'}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>
      
      <div className="radio-pill-group" id="formats-grid">
        {filteredFormats.map(f => {
          const ext = f.extension.replace(/^\./, '').toUpperCase();
          const isPopular = POPULAR_FORMATS.has(f.extension.toLowerCase().replace(/^\./, ''));
          const isSelected = formats.includes(f.extension.toLowerCase().replace(/^\./, ''));
          return (
            <label key={f.id} className={`radio-pill ${isSelected ? 'selected' : ''}`} title={`${f.name}${isPopular ? ' (Popular)' : ''}`}>
              <input
                type="checkbox"
                checked={isSelected}
                onChange={() => toggleFormat(f.id)}
                aria-label={f.name}
              />
              <span>{ext}</span>
            </label>
          );
        })}
      </div>
      
      {filteredFormats.length === 0 && (
        <div className="empty-state">
          <p>{t('no-formats-found') || 'Nenhum formato encontrado'}</p>
        </div>
      )}

      <button 
        type="button" 
        className={`btn-toggle-all-formats ${!showAll ? 'active' : ''}`}
        id="btn-toggle-all-formats"
        onClick={() => setShowAll(!showAll)}
      >
        {showAll ? (t('output.showPopular') || 'Mostrar Apenas Populares') : (t('output.showAll') || 'Mostrar Todos')}
      </button>
    </div>
  );
}
