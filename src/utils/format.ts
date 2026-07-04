/**
 * Trix Audio Converter � format
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * File size formatting utilities
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
const sizeFormatters: Record<string, Intl.NumberFormat> = {};

/**
 * Returns a cached `Intl.NumberFormat` for the given locale and decimal places.
 * Formatters are expensive to construct, so each unique `locale-decimals` pair
 * is memoised in `sizeFormatters` on first use.
 */
function getFormatter(locale: string, decimals: number): Intl.NumberFormat {
  const key = `${locale}-${decimals}`;
  if (!sizeFormatters[key]) {
    sizeFormatters[key] = new Intl.NumberFormat(locale, {
      minimumFractionDigits: 0,
      maximumFractionDigits: decimals,
    });
  }
  return sizeFormatters[key];
}

/**
 * Formats a byte count as a human-readable size string using locale-aware number formatting.
 *
 * @param bytes  - Raw byte count to format.
 * @param locale - BCP-47 locale string (defaults to `navigator.language` → `"pt-BR"`).
 * @returns A string like `"1.4 KB"`, `"23.5 MB"`, or `"1.20 GB"`.
 */
export function formatSize(bytes: number, locale?: string): string {
  const loc = locale || navigator.language || 'pt-BR';
  if (bytes < 1024) return getFormatter(loc, 0).format(bytes) + ' B';
  if (bytes < 1048576) return getFormatter(loc, 1).format(bytes / 1024) + ' KB';
  if (bytes < 1073741824) return getFormatter(loc, 1).format(bytes / 1048576) + ' MB';
  return getFormatter(loc, 2).format(bytes / 1073741824) + ' GB';
}
