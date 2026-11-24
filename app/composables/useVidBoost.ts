import { computed, reactive, ref, watch } from 'vue'
import {
  createDefaultSettings,
  formats,
  presets,
  JOB_STORAGE_KEY,
  LOG_STORAGE_KEY,
  SETTINGS_STORAGE_KEY,
  type AiModel,
  type Job,
  type LogEntry,
  type PreviewMode,
  type Settings,
  type StatusResponse,
  formatElapsed,
} from '~/utils/vidboost'
import { apiUrl, request } from '~/utils/api'

const MAX_FILE_SIZE = 400 * 1024 * 1024

const settings = reactive<Settings>(createDefaultSettings())
const job = ref<Job | null>(null)
const status = ref<StatusResponse | null>(null)
const statusLoading = ref(true)
const models = ref<AiModel[]>([])
const busy = ref(false)
const error = ref('')
const errorHidden = ref(false)
const isDragging = ref(false)
const previewMode = ref<PreviewMode>('after')
const split = ref(50)
const logEntries = ref<LogEntry[]>([])
const processStartedAt = ref<number | null>(null)
const elapsedSeconds = ref(0)

let apiBase = ''
let initialized = false
let pollTimer: ReturnType<typeof setInterval> | null = null
let noticeTimer: ReturnType<typeof setTimeout> | null = null

const api = (path: string): string => apiUrl(`${apiBase}${path}`)

const hasFile = computed(() => job.value !== null)
const isProcessing = computed(() => job.value?.status === 'processing')
const isDone = computed(() => job.value?.status === 'done')
const meta = computed(() => job.value?.meta ?? null)
const progressPct = computed(() => (isDone.value ? 100 : Math.round(job.value?.progress ?? 0)))
const etaSeconds = computed(() => {
  if (!isProcessing.value || progressPct.value <= 0 || elapsedSeconds.value < 1) return null
  return Math.max(0, Math.round(elapsedSeconds.value * (100 - progressPct.value) / progressPct.value))
})
const aiAvailable = computed(() => status.value?.ai.ok === true)

const aiBlockReason = computed(() => {
  const m = meta.value
  if (!m) return ''
  if (m.width > 1280 || m.height > 1280) return `Resolusi ${m.width}x${m.height} terlalu besar untuk AI (maks 720p).`
  if (m.fps > 30) return `${Math.round(m.fps)} fps terlalu tinggi untuk AI (maks 30 fps).`
  if (m.duration > 10) return `Durasi ${Math.round(m.duration)} detik terlalu panjang untuk AI (maks 10 detik).`
  return ''
})

const aiHeavy = computed(() => aiBlockReason.value !== '')

const outputPixelCap = computed(() => {
  const hasX264 = status.value?.encoders?.includes('libx264') ?? false
  const codecCap = !hasX264
    && ['auto', 'h264'].includes(settings.output.codec)
    && ['mp4', 'mov', 'mkv'].includes(settings.output.format)
    ? 9_200_000
    : Number.MAX_SAFE_INTEGER
  return Math.min(codecCap, status.value?.maxOutputPixels ?? 5_000_000)
})

const effectiveUpscaleFactor = computed(() => {
  const m = meta.value
  if (!m || !settings.upscale.enabled) return settings.upscale.factor
  const sourcePixels = m.width * m.height
  if (sourcePixels <= 0) return settings.upscale.factor
  const allowed = Math.sqrt(outputPixelCap.value / sourcePixels)
  return Math.max(1, Math.min(settings.upscale.factor, allowed))
})

const encodeWarnings = computed<string[]>(() => {
  const result: string[] = []
  const m = meta.value

  if (m && settings.upscale.enabled && !settings.upscale.ai && effectiveUpscaleFactor.value < settings.upscale.factor) {
    result.push(
      `Upscale ${settings.upscale.factor}x diturunkan jadi ~${effectiveUpscaleFactor.value.toFixed(2)}x. ` +
      `Batas output ${(outputPixelCap.value / 1_000_000).toFixed(0)} MP supaya video bisa diputar di HP.`,
    )
  }

  if (settings.upscale.ai && aiBlockReason.value) {
    result.push(aiBlockReason.value)
  }

  return result
})
const ffmpegMissing = computed(() => !statusLoading.value && status.value !== null && !status.value.ffmpeg.ok)
const displayError = computed(() => error.value || (job.value?.status === 'error' ? (job.value.error || 'Proses gagal. Coba lagi.') : ''))
const visibleError = computed(() => (errorHidden.value ? '' : displayError.value))

const codecOptions = computed(() =>
  settings.output.format === 'webm' ? ['auto', 'vp9', 'av1'] : ['auto', 'h264', 'hevc', 'av1'],
)

const presetOptions = presets.map((preset) => ({
  value: preset,
  label: preset === 'ultrafast' ? 'ultra' : preset === 'veryslow' ? 'very' : preset === 'medium' ? 'med' : preset,
}))

const modelOptions = computed<AiModel[]>(() => {
  if (models.value.length > 0) return models.value
  return [{
    id: 'fsrcnn',
    name: 'FSRCNN (CPU, ringan)',
    model: 'fsrcnn',
    scale: 4,
    runtime: 'opencv-cpu',
    size_mb: 1,
    vram: '0',
    ram: '~200 MB',
    speed: 'sedang',
    quality: 'sedang',
    pros: ['Paling ringan'],
    cons: ['Detail modest'],
    note: '',
    recommended: true,
  }]
})

const aiChipOptions = computed(() => modelOptions.value.map((model) => ({
  value: model.id,
  label: model.name,
  hint: typeof model.ram === 'string' && model.ram.includes('MB') ? model.ram : undefined,
})))

const selectedModelLabel = computed(() =>
  models.value.find((item) => item.id === settings.upscale.aiModel)?.name ?? settings.upscale.aiModel,
)

const sourceUrl = computed(() => (job.value ? api(`/api/video/source?id=${encodeURIComponent(job.value.id)}`) : ''))
const downloadUrl = computed(() => (job.value ? api(`/api/video/download?id=${encodeURIComponent(job.value.id)}`) : ''))
const thumbUrl = computed(() => (job.value ? api(`/api/video/thumbnail?id=${encodeURIComponent(job.value.id)}`) : ''))
const afterSource = computed(() => (isDone.value ? `${downloadUrl.value}&inline=1` : sourceUrl.value))

const chips = computed<string[]>(() => {
  const result: string[] = []
  if (settings.upscale.enabled) {
    const factor = settings.upscale.ai ? settings.upscale.factor : effectiveUpscaleFactor.value
    const label = Number(factor.toFixed(2)).toString()
    result.push(`${label}x ${settings.upscale.ai ? selectedModelLabel.value : 'Lanczos'}`)
  }
  if (settings.detail.enabled && settings.detail.sharpen > 0) result.push(`Sharpen ${settings.detail.sharpen}`)
  if (settings.detail.enabled && settings.detail.denoise > 0) result.push(`Denoise ${settings.detail.denoise}`)
  if (settings.geometry.rotate !== 0) result.push(`Rotate ${settings.geometry.rotate}°`)
  if (settings.geometry.cropEnabled) result.push('Crop')
  if (settings.watermark.mode !== 'none') result.push(`Watermark: ${settings.watermark.mode === 'text' ? 'teks' : 'gambar'}`)
  result.push(`${settings.output.format.toUpperCase()} · ${settings.output.codec === 'auto' ? 'Auto' : settings.output.codec.toUpperCase()} · CRF ${settings.output.crf}`)
  if (!settings.output.audio) result.push('Tanpa audio')
  return result
})

watch(aiAvailable, (available) => {
  if (!available) settings.upscale.ai = false
})

watch(aiHeavy, (heavy) => {
  if (heavy) settings.upscale.ai = false
})

watch(() => settings.output.format, () => {
  if (!codecOptions.value.includes(settings.output.codec)) settings.output.codec = 'auto'
})

watch(visibleError, (value) => {
  clearNoticeTimer()
  if (value) {
    noticeTimer = setTimeout(() => {
      errorHidden.value = true
      clearNoticeTimer()
    }, 9000)
  }
})

watch(settings, () => {
  if (typeof localStorage !== 'undefined' && job.value) {
    localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings))
  }
}, { deep: true })

let elapsedTimer: ReturnType<typeof setInterval> | null = null
let logSeq = 0
let lastLogKey = ''

function stageKey(stage: string, progress: number): string {
  const normalized = stage.replace(/[0-9]+([.,][0-9]+)?/g, '#')
  return `${normalized}|${Math.floor(progress / 5)}`
}

function appendLog(text: string, tone: LogEntry['tone'] = 'info') {
  const time = new Date().toLocaleTimeString('id-ID', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
  logEntries.value.push({ id: ++logSeq, time, text, tone })
  if (logEntries.value.length > 50) {
    logEntries.value.splice(0, logEntries.value.length - 50)
  }
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(LOG_STORAGE_KEY, JSON.stringify(logEntries.value))
  }
}

function resetLog() {
  logEntries.value = []
  lastLogKey = ''
  if (typeof localStorage !== 'undefined') localStorage.removeItem(LOG_STORAGE_KEY)
}

function stopElapsedTimer() {
  if (elapsedTimer !== null) {
    clearInterval(elapsedTimer)
    elapsedTimer = null
  }
}

function startElapsedTimer() {
  stopElapsedTimer()
  processStartedAt.value = Date.now()
  elapsedSeconds.value = 0
  elapsedTimer = setInterval(() => {
    if (processStartedAt.value === null) return
    elapsedSeconds.value = Math.floor((Date.now() - processStartedAt.value) / 1000)
  }, 1000)
}

function disposeVidBoost() {
  // Jangan matikan polling kalau proses masih jalan saat pindah halaman.
  if (!isProcessing.value) stopPolling()
  stopElapsedTimer()
}

function stopPolling() {
  if (pollTimer !== null) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

function startPolling(id: string) {
  stopPolling()
  pollTimer = setInterval(async () => {
    try {
      const fresh = await request<Job>(api(`/api/video/job?id=${encodeURIComponent(id)}`))
      const previousStatus = job.value?.status
      job.value = fresh
      errorHidden.value = false

      const key = `${fresh.status}|${stageKey(fresh.stage, fresh.progress)}`
      if (key !== lastLogKey) {
        lastLogKey = key
        const tone: LogEntry['tone'] =
          fresh.status === 'error' ? 'error'
            : fresh.status === 'done' ? 'success'
              : 'info'
        const text = fresh.status === 'done'
          ? (fresh.stage || 'Selesai')
          : fresh.status === 'error'
            ? (fresh.error || 'Proses gagal')
            : (fresh.stage || 'Menyiapkan…')
        appendLog(text, tone)
      }
      if (previousStatus !== 'processing' && fresh.status === 'processing') {
        startElapsedTimer()
      }
      if (fresh.status === 'done' || fresh.status === 'error') {
        stopPolling()
        stopElapsedTimer()
      }
    } catch {
      // Transient network failure: keep polling.
    }
  }, 800)
}

function clearNoticeTimer() {
  if (noticeTimer !== null) {
    clearTimeout(noticeTimer)
    noticeTimer = null
  }
}

function dismissError() {
  errorHidden.value = true
  clearNoticeTimer()
}

function validateFile(file: File): string | null {
  const isVideo = file.type.startsWith('video/') || /\.(mp4|webm|mov|mkv|avi|m4v)$/i.test(file.name)
  if (!isVideo) return 'Pilih file video (mp4, webm, mov, mkv, atau avi).'
  if (file.size > MAX_FILE_SIZE) return 'Ukuran file melebihi 400 MB.'
  return null
}

async function uploadVideo(file: File) {
  errorHidden.value = false
  const problem = validateFile(file)
  if (problem) {
    error.value = problem
    return
  }
  error.value = ''
  busy.value = true
  try {
    const form = new FormData()
    form.append('file', file)
    const created = await request<Job>(api('/api/video/upload'), { method: 'POST', body: form })
    stopPolling()
    job.value = created
    previewMode.value = 'after'
    errorHidden.value = false
    if (created.status === 'processing') startPolling(created.id)
  } catch {
    error.value = 'Upload gagal. Periksa koneksi ke server lokal, lalu coba lagi.'
  } finally {
    busy.value = false
  }
}

async function uploadLogo(file: File) {
  errorHidden.value = false
  if (!job.value) return
  if (!file.type.startsWith('image/')) {
    error.value = 'File logo harus berupa gambar.'
    return
  }
  error.value = ''
  busy.value = true
  try {
    const form = new FormData()
    form.append('file', file)
    const result = await request<{ path: string }>(
      api(`/api/video/logo?id=${encodeURIComponent(job.value.id)}`),
      { method: 'POST', body: form },
    )
    settings.watermark.logoPath = result.path
  } catch {
    error.value = 'Upload logo gagal. Coba lagi.'
  } finally {
    busy.value = false
  }
}

async function startProcess() {
  if (!job.value || busy.value || isProcessing.value) return
  errorHidden.value = false
  error.value = ''
  busy.value = true
  resetLog()
  startElapsedTimer()
  appendLog('Proses dimulai')
  try {
    const result = await request<{ ok: boolean, id: string }>(api('/api/video/process'), {
      method: 'POST',
      body: { id: job.value.id, settings: JSON.parse(JSON.stringify(settings)) as Settings },
    })
    if (!result.ok) throw new Error('Process rejected')
    job.value = { ...job.value, status: 'processing', progress: 0, stage: 'Menyiapkan', error: null }
    previewMode.value = 'after'
    startPolling(result.id)
  } catch {
    error.value = 'Proses gagal dimulai. Periksa status FFmpeg, lalu coba lagi.'
    appendLog(error.value, 'error')
    stopElapsedTimer()
  } finally {
    busy.value = false
  }
}

async function cancelJob() {
  if (!job.value) return
  try {
    await $fetch(api(`/api/video/cancel?id=${encodeURIComponent(job.value.id)}`), { method: 'POST' })
  } catch {
    // Server might already have finished; local state still reflects the cancel intent.
  }
  stopPolling()
  stopElapsedTimer()
  appendLog('Dibatalkan oleh pengguna', 'error')
  job.value = { ...job.value, status: 'error', error: 'Dibatalkan oleh pengguna', stage: 'Dibatalkan' }
}

function resetColor() {
  Object.assign(settings.color, createDefaultSettings().color)
}

function resetAll() {
  stopPolling()
  stopElapsedTimer()
  clearNoticeTimer()
  resetLog()
  processStartedAt.value = null
  elapsedSeconds.value = 0
  job.value = null
  error.value = ''
  errorHidden.value = false
  previewMode.value = 'after'
  split.value = 50
  Object.assign(settings, createDefaultSettings())
  if (typeof localStorage !== 'undefined') {
    localStorage.removeItem(JOB_STORAGE_KEY)
    localStorage.removeItem(SETTINGS_STORAGE_KEY)
  }
}

function persistForExport() {
  if (!job.value || typeof localStorage === 'undefined') return
  localStorage.setItem(JOB_STORAGE_KEY, job.value.id)
  localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings))
}

async function initVidBoost() {
  if (initialized) return
  initialized = true
  apiBase = (useRuntimeConfig().public.apiBase as string) || ''

  const [statusResult, modelsResult] = await Promise.allSettled([
    request<StatusResponse>(api('/api/status')),
    request<AiModel[]>(api('/api/ai/models')),
  ])
  if (statusResult.status === 'fulfilled') {
    status.value = statusResult.value
  } else {
    status.value = { ffmpeg: { ok: false, version: '' }, ai: { ok: false, path: '', binary: '' }, encoders: [], maxOutputPixels: 5_000_000, ffmpegThreads: '2', dataDir: '' }
  }
  if (modelsResult.status === 'fulfilled' && Array.isArray(modelsResult.value)) {
    models.value = modelsResult.value
  }
  statusLoading.value = false

  if (typeof localStorage === 'undefined') return
  const savedSettings = localStorage.getItem(SETTINGS_STORAGE_KEY)
  if (savedSettings) {
    try {
      Object.assign(settings, JSON.parse(savedSettings))
    } catch {
      // Ignore malformed saved settings.
    }
  }
  const savedLog = localStorage.getItem(LOG_STORAGE_KEY)
  if (savedLog) {
    try {
      const parsed = JSON.parse(savedLog) as LogEntry[]
      if (Array.isArray(parsed)) {
        logEntries.value = parsed.slice(-50)
        logSeq = parsed.reduce((max, entry) => Math.max(max, entry.id), 0)
      }
    } catch {
      localStorage.removeItem(LOG_STORAGE_KEY)
    }
  }

  const savedJobId = localStorage.getItem(JOB_STORAGE_KEY)
  if (savedJobId && !job.value) {
    try {
      job.value = await request<Job>(api(`/api/video/job?id=${encodeURIComponent(savedJobId)}`))
      if (job.value.status === 'processing') startPolling(job.value.id)
    } catch {
      localStorage.removeItem(JOB_STORAGE_KEY)
    }
  }
}

export function useVidBoost() {
  return {
    settings,
    job,
    status,
    statusLoading,
    models,
    busy,
    error,
    errorHidden,
    isDragging,
    previewMode,
    split,
    logEntries,
    elapsedSeconds,
    etaSeconds,
    formatElapsed,
    disposeVidBoost,
    hasFile,
    isProcessing,
    isDone,
    meta,
    progressPct,
    aiAvailable,
    aiHeavy,
    aiBlockReason,
    encodeWarnings,
    effectiveUpscaleFactor,
    outputPixelCap,
    ffmpegMissing,
    displayError,
    visibleError,
    codecOptions,
    presetOptions,
    formats,
    modelOptions,
    aiChipOptions,
    selectedModelLabel,
    sourceUrl,
    downloadUrl,
    thumbUrl,
    afterSource,
    chips,
    initVidBoost,
    uploadVideo,
    uploadLogo,
    startProcess,
    cancelJob,
    resetColor,
    resetAll,
    persistForExport,
    dismissError,
    startPolling,
    stopPolling,
    clearNoticeTimer,
  }
}
