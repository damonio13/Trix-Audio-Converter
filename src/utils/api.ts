/**
 * Trix Audio Converter � api
 *
 * @author Jo�o Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
/**
 * HTTP API client for backend communication
 *
 * @author João Vitor de Melo <joaovmelo259@gmail.com>
 * @version 1.0.0
 * @license MIT
 */
import { FileItem, ConvertOptions } from '@/types';

let _apiToken = '';

/**
 * Loads the API token from the WebView initialization context at startup.
 *
 * Priority order:
 * 1. `window.__API_TOKEN` — injected by the Rust backend into the WebView.
 * 2. `<meta name="api-token">` — legacy HTML fallback.
 * 3. `#__api_token_data` element — DOM-based fallback.
 * 4. `sessionStorage.__trix_api_token` — persisted across soft reloads.
 *
 * Each source is deleted immediately after reading to minimize token exposure.
 */
(function loadToken() {
  try {
    const w = window as unknown as { __API_TOKEN?: string };
    if (w.__API_TOKEN) {
      _apiToken = w.__API_TOKEN;
      try { sessionStorage.setItem('__trix_api_token', _apiToken); } catch { /* ignore */ }
      delete w.__API_TOKEN;
      return;
    }
  } catch { /* window or API_TOKEN unavailable */ }

  try {
    const meta = document.querySelector('meta[name="api-token"]');
    if (meta) {
      _apiToken = meta.getAttribute('content') || '';
      meta.remove();
      try { sessionStorage.setItem('__trix_api_token', _apiToken); } catch { /* ignore */ }
      return;
    }
  } catch { /* meta tag not found or DOM unavailable */ }

  try {
    const el = document.getElementById('__api_token_data');
    if (el) {
      _apiToken = el.textContent || '';
      el.remove();
      try { sessionStorage.setItem('__trix_api_token', _apiToken); } catch { /* ignore */ }
      return;
    }
  } catch { /* element not found */ }

  try {
    const storedToken = sessionStorage.getItem('__trix_api_token');
    if (storedToken) {
      _apiToken = storedToken;
      return;
    }
  } catch { /* sessionStorage unavailable */ }
})();

/**
 * Singleton HTTP client for communicating with the Rust Axum backend.
 *
 * All requests are authenticated with a `Bearer` token resolved at startup.
 * The base URL and token are read from `window.__API_URL` / `window.__API_TOKEN`
 * (injected by the backend) with a sessionStorage fallback for soft reloads.
 */
class ApiClient {
  private get baseUrl() {
    const w = window as unknown as { __API_URL?: string };
    let url = w.__API_URL || '';
    if (url) {
      try { sessionStorage.setItem('__trix_api_url', url); } catch { /* ignore */ }
    } else {
      try { url = sessionStorage.getItem('__trix_api_url') || ''; } catch { /* ignore */ }
    }
    return url + '/api';
  }

  private get apiToken() {
    const w = window as unknown as { __API_TOKEN?: string };
    if (w.__API_TOKEN) return w.__API_TOKEN;
    return _apiToken;
  }

  /**
   * Generic fetch wrapper that appends the `Authorization` header and parses
   * the JSON response. Throws a user-friendly error on network failure,
   * `AbortError` (timeout), or non-2xx status codes.
   */
  async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    const token = this.apiToken;
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    if (options.headers) {
      Object.assign(headers, options.headers);
    }

    let response: Response;
    try {
      response = await fetch(`${this.baseUrl}${endpoint}`, {
        ...options,
        headers,
      });
    } catch (err) {
      if (err instanceof TypeError && err.message === 'Failed to fetch') {
        // eslint-disable-next-line preserve-caught-error
        throw new Error('Sem conexão com o servidor. Verifique se o backend está rodando.');
      }
      if (err instanceof DOMException && err.name === 'AbortError') {
        // eslint-disable-next-line preserve-caught-error
        throw new Error(`Timeout na requisição para ${endpoint}`);
      }
      // eslint-disable-next-line preserve-caught-error
      throw new Error(`Erro de rede: ${(err as Error).message || 'desconhecido'}`);
    }

    if (!response.ok) {
      throw new Error(`API error: ${response.status} ${response.statusText}`);
    }

    const text = await response.text();
    if (!text) {
      throw new Error(`Empty response from ${endpoint}`);
    }
    try {
      return JSON.parse(text) as T;
    } catch {
      throw new Error(`Invalid JSON response from ${endpoint}`);
    }
  }

  /** Sends a `GET` request to `endpoint` and returns the parsed JSON body. */
  get<T>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'GET' });
  }

  /** Sends a `POST` request with `body` serialised as JSON. */
  post<T>(endpoint: string, body: unknown): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: JSON.stringify(body),
    });
  }

  /** Returns all 106 supported output formats keyed by format string (e.g. `".mp3"`). */
  async getFormats() {
    return this.get<{ [key: string]: { ext?: string; cat?: string } }>('/formats');
  }

  /**
   * Scans the given file system paths for supported audio/video files.
   * Individual files are added directly; directories are scanned recursively.
   */
  async scanFolders(paths: string[]) {
    return this.post<FileItem[]>('/scan', { paths });
  }

  /** Opens the OS native file-picker dialog and returns the selected paths. */
  async pickFiles() {
    return this.post<string[]>('/pick-files', {});
  }

  /** Drains the list of files drag-and-dropped onto the native window. */
  async getDroppedFiles() {
    return this.get<string[]>('/dropped-files');
  }

  /** Starts the conversion job. Returns a `jobId` to poll with {@link getConversionStatus}. */
  async startConversion(options: Partial<ConvertOptions> & { files?: string[] }) {
    return this.post<{ jobId: string }>('/start', options);
  }

  /** Polls the real-time status of a running conversion job. */
  async getConversionStatus(jobId: string) {
    return this.get<{ converting: boolean; progress: number; converted: number; files: Record<string, { status: string; progress: number; error?: string; input?: string; filename?: string }> }>(`/status/${jobId}`);
  }

  /** Opens a folder in the OS file explorer, or shows a folder-picker when `path` is empty. */
  async openFolder(path: string) {
    return this.post<{ success: boolean; error?: string }>('/open-folder', { path });
  }

  /**
   * Two-phase cache cleanup.
   * - `confirm: false` — dry run, returns the file count.
   * - `confirm: true`  — actually deletes temp files.
   */
  async clearCache(confirm: boolean) {
    return this.post<{ success: boolean; requires_confirmation?: boolean; files_to_delete?: number; removed?: number }>('/clear-cache', { confirm });
  }

  /** Registers the "Convert with Trix Audio" Windows shell context-menu entry. */
  async registerContextMenu() {
    return this.post<{ success: boolean; error?: string }>('/context-menu/register', {});
  }

  /** Removes the "Convert with Trix Audio" Windows shell context-menu entry. */
  async unregisterContextMenu() {
    return this.post<{ success: boolean; error?: string }>('/context-menu/unregister', {});
  }

  /** Sends a close command to the native tao window. */
  async windowClose() {
    return this.post<{ success: boolean }>('/window/close', {});
  }

  /** Sends a minimize command to the native tao window. */
  async windowMinimize() {
    return this.post<{ success: boolean }>('/window/minimize', {});
  }

  /** Sends a maximize/restore command to the native tao window. */
  async windowMaximize() {
    return this.post<{ success: boolean }>('/window/maximize', {});
  }

  /** Returns metadata for all crash log files stored in the app logs directory. */
  async getCrashLogs() {
    return this.get<{ success: boolean; logs: { name: string; path: string; size: number; date: string }[] }>('/crash-logs');
  }

  /** Reads the full text content of a crash log file by absolute path. */
  async readCrashLog(path: string) {
    return this.post<{ success: boolean; content: string }>('/crash-logs/read', { path });
  }

  /** Permanently deletes a crash log file by absolute path. */
  async deleteCrashLog(path: string) {
    return this.post<{ success: boolean; error?: string }>('/crash-logs/delete', { path });
  }
}

export const api = new ApiClient();
