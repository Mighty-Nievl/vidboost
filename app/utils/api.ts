/** Satu titik akses API. Web memakai $fetch; Tauri sementara memakai sidecar
 * di 127.0.0.1:19090. Saat build Android, modul ini akan dialihkan ke invoke. */
export function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export function isTauriMobile(): boolean {
  return isTauriRuntime() && /Android|iPhone|iPad/i.test(navigator.userAgent)
}

export function apiUrl(path: string): string {
  if (typeof window === 'undefined') return path
  if (isTauriRuntime() && !path.startsWith('http')) {
    return `http://127.0.0.1:19090${path}`
  }
  return path
}

export async function request<T>(path: string, options?: Parameters<typeof $fetch>[1]): Promise<T> {
  return $fetch<T>(apiUrl(path), options)
}
