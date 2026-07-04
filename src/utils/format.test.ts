/**
 * Trix Audio Converter � format.test
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { describe, it, expect } from 'vitest';
import { formatSize } from './format';

describe('formatSize', () => {
  it('formats bytes correctly', () => {
    expect(formatSize(500, 'pt-BR')).toBe('500 B');
  });

  it('formats kilobytes correctly', () => {
    expect(formatSize(1024, 'pt-BR')).toBe('1 KB');
    expect(formatSize(1536, 'pt-BR')).toBe('1,5 KB');
  });

  it('formats megabytes correctly', () => {
    expect(formatSize(1048576, 'pt-BR')).toBe('1 MB');
    expect(formatSize(1572864, 'pt-BR')).toBe('1,5 MB');
  });

  it('formats gigabytes correctly', () => {
    expect(formatSize(1073741824, 'pt-BR')).toBe('1 GB');
    expect(formatSize(1610612736, 'pt-BR')).toBe('1,5 GB');
  });
});
