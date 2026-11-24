export type JobStatus = 'ready' | 'processing' | 'done' | 'error'
export type WatermarkPosition = 'top-left' | 'top-right' | 'center' | 'bottom-left' | 'bottom-right'
export type PreviewMode = 'before' | 'after' | 'split'
export type TabId = 'source' | 'color' | 'detail' | 'upscale' | 'crop' | 'text' | 'export'

export interface LogEntry {
  id: number
  time: string
  text: string
  tone: 'info' | 'success' | 'error'
}

export function formatElapsed(seconds: number): string {
  const total = Math.max(0, Math.floor(seconds))
  return `${String(Math.floor(total / 60)).padStart(2, '0')}:${String(total % 60).padStart(2, '0')}`
}

export interface JobMeta {
  width: number
  height: number
  duration: number
  fps: number
  codec: string
  bitrate: number
  mime: string
}

export interface Job {
  id: string
  status: JobStatus
  progress: number
  stage: string
  error: string | null
  fileName: string
  size: number
  meta: JobMeta | null
  outputName: string | null
  createdAt: number
  updatedAt: number
}

export interface StatusResponse {
  ffmpeg: { ok: boolean, version: string }
  ai: { ok: boolean, path: string, binary: string }
  encoders: string[]
  maxOutputPixels: number
  ffmpegThreads: string
  dataDir: string
}

export interface AiModel {
  id: string
  name: string
  model: string
  scale: number
  runtime: string
  size_mb: number
  vram: string
  ram: string
  speed: string
  quality: string
  pros: string[]
  cons: string[]
  note: string
  recommended: boolean
}

export interface Settings {
  color: { enabled: boolean, brightness: number, contrast: number, saturation: number, gamma: number }
  detail: { enabled: boolean, sharpen: number, denoise: number }
  upscale: { enabled: boolean, factor: number, ai: boolean, aiModel: string }
  geometry: { rotate: number, cropEnabled: boolean, cropX: number, cropY: number, cropW: number, cropH: number }
  watermark: {
    mode: 'none' | 'text' | 'image'
    text: string
    fontSize: number
    color: string
    position: WatermarkPosition
    logoPath: string | null
    opacity: number
    scale: number
  }
  output: { format: string, codec: string, crf: number, preset: string, audio: boolean }
}

export function createDefaultSettings(): Settings {
  return {
    color: { enabled: true, brightness: 0, contrast: 1, saturation: 1, gamma: 1 },
    detail: { enabled: true, sharpen: 25, denoise: 0 },
    upscale: { enabled: false, factor: 2, ai: false, aiModel: 'fsrcnn' },
    geometry: { rotate: 0, cropEnabled: false, cropX: 0, cropY: 0, cropW: 100, cropH: 100 },
    watermark: { mode: 'none', text: '', fontSize: 48, color: '#ffffff', position: 'bottom-right', logoPath: null, opacity: 1, scale: 20 },
    output: { format: 'mp4', codec: 'auto', crf: 23, preset: 'veryfast', audio: true },
  }
}

export const formats = ['mp4', 'webm', 'mov', 'mkv']
export const presets = ['ultrafast', 'fast', 'medium', 'slow', 'veryslow']

export const factorOptions = [
  { value: '1', label: '1x' },
  { value: '2', label: '2x' },
  { value: '3', label: '3x' },
  { value: '4', label: '4x' },
]

export const rotateOptions = [
  { value: '0', label: '0°' },
  { value: '90', label: '90°' },
  { value: '180', label: '180°' },
  { value: '270', label: '270°' },
]

export const watermarkModes = [
  { value: 'none', label: 'Tanpa' },
  { value: 'text', label: 'Teks' },
  { value: 'image', label: 'Gambar' },
]

export const positionOptions = [
  { value: 'top-left', label: 'Kiri atas' },
  { value: 'top-right', label: 'Kanan atas' },
  { value: 'center', label: 'Tengah' },
  { value: 'bottom-left', label: 'Kiri bawah' },
  { value: 'bottom-right', label: 'Kanan bawah' },
]

export const JOB_STORAGE_KEY = 'vidboost:jobId'
export const SETTINGS_STORAGE_KEY = 'vidboost:settings'
export const LOG_STORAGE_KEY = 'vidboost:log'
export const THEME_STORAGE_KEY = 'vidboost:theme'

export function formatSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 KB'
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

export function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return '0:00'
  const total = Math.floor(seconds)
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`
}
