<script setup lang="ts">
type BootstrapPreset = {
  id: string
  label: string
  description: string
  qualityRange: string
  sizeReductionRange: string
}

type BootstrapCapability = {
  kind: string
  compressionModes: string[]
  conversions: string[]
}

type FormatTargets = {
  kind: string
  targets: string[]
}

type BootstrapData = {
  appName: string
  version: string
  summary: string
  presets: BootstrapPreset[]
  mediaCapabilities: BootstrapCapability[]
  formatTargets: FormatTargets[]
  gifWorkflow: string[]
}

type AppUpdateStatus = {
  configured: boolean
  currentVersion: string
  availableVersion: string | null
  notes: string | null
  pubDate: string | null
  updateReady: boolean
  updateInstalled: boolean
  message: string
}

type BatchJobResult = {
  jobId: string
  label: string | null
  inputPath: string
  mediaKind: string
  operation: string
  outputPath: string | null
  success: boolean
  skipped: boolean
  cancelled: boolean
  ffmpegArgs: string[]
  message: string
}

type BatchProgressEvent = {
  jobId: string
  label: string | null
  inputPath: string
  mediaKind: string
  operation: string
  status: string
  progressPercent: number | null
  outputPath: string | null
  message: string
  speed: string | null
}

type ResourceJobEstimate = {
  jobId: string
  label: string
  mediaKind: string
  operation: string
  estimatedMemoryMb: number
  estimatedSeconds: number
}

type ResourcePlan = {
  logicalCores: number
  availableMemoryMb: number | null
  totalMemoryMb: number | null
  maxParallelJobs: number
  safeParallelJobs: number
  estimatedParallelMemoryMb: number
  estimatedTotalSeconds: number
  estimatedParallelSeconds: number
  canRunInParallel: boolean
  shouldUseSequential: boolean
  summary: string
  jobs: ResourceJobEstimate[]
}

type LiveSystemMetrics = {
  cpuUsagePercent: number
  usedMemoryMb: number
  availableMemoryMb: number
  totalMemoryMb: number
}

type QueueProgress = {
  jobId: string
  label: string | null
  mediaKind: string
  operation: string
  status: string
  progressPercent: number
  outputPath: string | null
  message: string
  speed: string | null
}

type GifSegment = {
  jobId: string
  inputPath: string
  outputSuffix: string
  label: string
  startSeconds: number
  durationSeconds: number
  fps: number
  width: number
}

type MixedJob = {
  jobId: string
  label: string
  inputPath: string
  outputDir: string
  mode: string
  presetId: string
  videoFormat: string
  imageFormat: string
  audioFormat: string
  resizeLongEdge: number | null
  gif: {
    startSeconds: number
    durationSeconds: number
    fps: number
    width: number
  } | null
  outputSuffix: string | null
  overwrite: boolean
}

type QueueItem = string | GifSegment

const modeOptions = [
  { label: 'Compress', value: 'compress' },
  { label: 'Convert', value: 'convert' },
  { label: 'Create GIF', value: 'gif' }
]

const files = ref<string[]>([])
const outputDir = ref('')
const mode = ref('compress')
const presetId = ref('balanced')
const resizeLongEdge = ref(1280)
const maxParallelJobs = ref(2)
const videoFormat = ref('mp4')
const imageFormat = ref('webp')
const audioFormat = ref('mp3')
const gifOptions = reactive({
  startSeconds: 0,
  durationSeconds: 3,
  fps: 12,
  width: 480
})
const results = ref<BatchJobResult[]>([])
const queueProgress = ref<Record<string, QueueProgress>>({})
const gifSegments = ref<GifSegment[]>([])
const activityQueue = ref<MixedJob[]>([])
const selectedGifVideo = ref('')
const selectedGifVideoSrc = ref('')
const selectedGifVideoDuration = ref(0)
const gifPreviewError = ref('')
const gifPreviewVideo = ref<HTMLVideoElement | null>(null)
const processing = ref(false)
const cancelPending = ref(false)
const currentRunId = ref('')
const bootstrap = ref<BootstrapData | null>(null)
const updateStatus = ref<AppUpdateStatus | null>(null)
const updateLoading = ref(false)
const updateInstalling = ref(false)
const updateToastId = ref<string | number | null>(null)
const resourcePlan = ref<ResourcePlan | null>(null)
const resourcePlanLoading = ref(false)
const liveSystemMetrics = ref<LiveSystemMetrics | null>(null)
const toast = useToast()
const activePreset = computed(() => bootstrap.value?.presets.find(preset => preset.id === presetId.value))
const videoFiles = computed(() => files.value.filter(path => detectKind(path) === 'video'))
const videoFileOptions = computed(() =>
  videoFiles.value.map(path => ({
    label: basename(path),
    value: path
  }))
)
const nonVideoFiles = computed(() => files.value.filter(path => detectKind(path) !== 'video'))
const gifQueue = computed(() => gifSegments.value)
const runQueue = computed<QueueItem[] | MixedJob[]>(() => {
  if (activityQueue.value.length) {
    return activityQueue.value
  }

  return mode.value === 'gif' ? gifQueue.value : files.value
})
const runQueueCount = computed(() => runQueue.value.length)
const effectiveParallelJobs = computed(() => Math.max(maxParallelJobs.value || 1, 1))
const gifClipRangeLabel = computed(() => {
  const start = gifOptions.startSeconds
  const end = gifOptions.startSeconds + gifOptions.durationSeconds
  return `${start.toFixed(1)}s -> ${end.toFixed(1)}s`
})
const gifEndSeconds = computed({
  get() {
    return Number((gifOptions.startSeconds + gifOptions.durationSeconds).toFixed(2))
  },
  set(value: number) {
    const maxEnd = selectedGifVideoDuration.value || value
    const clampedEnd = Math.min(Math.max(value, gifOptions.startSeconds + 0.5), maxEnd)
    gifOptions.durationSeconds = Number((clampedEnd - gifOptions.startSeconds).toFixed(2))
  }
})
const canRun = computed(() => {
  if (processing.value || !outputDir.value.length) {
    return false
  }

  if (resourcePlan.value?.shouldUseSequential && effectiveParallelJobs.value > 1) {
    return false
  }

  if (activityQueue.value.length) {
    return true
  }

  if (mode.value === 'gif') {
    return gifQueue.value.length > 0
  }

  return files.value.length > 0
})
const overallProgress = computed(() => {
  if (!runQueue.value.length) {
    return 0
  }
  const progressKeys = activityQueue.value.length
    ? activityQueue.value.map(job => job.jobId)
    : mode.value === 'gif'
      ? gifQueue.value.map(segment => segment.jobId)
      : files.value.map(path => batchJobId(path, mode.value))
  if (!progressKeys.length) {
    return 0
  }
  const total = progressKeys.reduce((sum, key) => sum + (queueProgress.value[key]?.progressPercent ?? 0), 0)
  return Math.round(total / progressKeys.length)
})
const completedJobs = computed(() =>
  Object.values(queueProgress.value).filter(item => isTerminalStatus(item.status)).length
)
const remainingJobCount = computed(() => {
  const estimateJobs = resourcePlan.value?.jobs ?? []
  return estimateJobs.filter((job) => {
    const status = queueProgress.value[job.jobId]?.status
    return !isTerminalStatus(status)
  }).length
})
const estimatedRemainingSeconds = computed(() => {
  const plan = resourcePlan.value
  if (!plan) {
    return 0
  }

  if (!processing.value) {
    return plan.estimatedParallelSeconds ?? 0
  }

  const jobs = plan.jobs ?? []
  if (!jobs.length) {
    return 0
  }

  const remainingWorkSeconds = jobs.reduce((sum, job) => {
    const progress = queueProgress.value[job.jobId]
    if (!progress || progress.status === 'queued') {
      return sum + job.estimatedSeconds
    }

    if (isTerminalStatus(progress.status)) {
      return sum
    }

    const ratio = Math.min(Math.max(progress.progressPercent, 0), 100) / 100
    return sum + (job.estimatedSeconds * (1 - ratio))
  }, 0)

  const parallelSlots = Math.max(Math.min(effectiveParallelJobs.value, remainingJobCount.value || 1), 1)
  return Math.ceil(remainingWorkSeconds / parallelSlots)
})
const estimatedMinutesLabel = computed(() => {
  const seconds = estimatedRemainingSeconds.value
  if (!seconds) {
    return 'n/a'
  }

  if (seconds < 60) {
    return `${seconds}s`
  }

  return `${Math.ceil(seconds / 60)} min`
})
const etaCaption = computed(() => processing.value ? 'Remaining ETA' : 'Planned ETA')

let unlistenBatchProgress: null | (() => void) = null
let liveMetricsInterval: ReturnType<typeof setInterval> | null = null

function basename(path: string) {
  return path.split(/[\\/]/).pop() || path
}

function detectKind(path: string) {
  const extension = path.split('.').pop()?.toLowerCase() || ''

  if (['mp4', 'mov', 'mkv', 'avi', 'webm'].includes(extension)) {
    return 'video'
  }

  if (['png', 'jpg', 'jpeg', 'webp', 'bmp', 'tiff'].includes(extension)) {
    return 'image'
  }

  if (['mp3', 'wav', 'aac', 'm4a', 'flac', 'opus', 'ogg'].includes(extension)) {
    return 'audio'
  }

  return 'unknown'
}

function targetFormats(kind: string) {
  return bootstrap.value?.formatTargets.find(target => target.kind === kind)?.targets ?? []
}

function batchJobId(path: string, operation: string) {
  return `${operation}::${path}`
}

function queueItemKey(item: QueueItem) {
  return typeof item === 'string' ? batchJobId(item, mode.value) : item.jobId
}

function queueItemProgress(item: QueueItem) {
  return queueProgress.value[queueItemKey(item)]
}

function mixedJobProgress(item: MixedJob) {
  return queueProgress.value[item.jobId]
}

function describeActivity(job: MixedJob) {
  if (job.mode === 'gif' && job.gif) {
    const end = job.gif.startSeconds + job.gif.durationSeconds
    return `${job.gif.startSeconds.toFixed(1)}s -> ${end.toFixed(1)}s • ${job.gif.fps} fps • ${job.gif.width}px`
  }

  const target = detectKind(job.inputPath) === 'video'
    ? job.videoFormat
    : detectKind(job.inputPath) === 'image'
      ? job.imageFormat
      : job.audioFormat
  const resize = job.resizeLongEdge ? ` • ${job.resizeLongEdge}px edge` : ''
  return `${job.mode} -> ${target}${resize}`
}

function clampGifRange() {
  if (!selectedGifVideoDuration.value) {
    return
  }

  gifOptions.startSeconds = Number(
    Math.min(Math.max(gifOptions.startSeconds, 0), Math.max(selectedGifVideoDuration.value - 0.5, 0)).toFixed(2)
  )

  const end = Math.min(gifOptions.startSeconds + gifOptions.durationSeconds, selectedGifVideoDuration.value)
  gifOptions.durationSeconds = Number(Math.max(end - gifOptions.startSeconds, 0.5).toFixed(2))
}

function setGifStart(value: number) {
  const maxStart = Math.max(selectedGifVideoDuration.value - 0.5, 0)
  gifOptions.startSeconds = Number(Math.min(Math.max(value, 0), maxStart).toFixed(2))
  if (gifEndSeconds.value <= gifOptions.startSeconds) {
    gifEndSeconds.value = gifOptions.startSeconds + 0.5
  }
  syncPreviewToGifStart()
}

function setGifEnd(value: number) {
  gifEndSeconds.value = value
}

function syncPreviewToGifStart() {
  if (gifPreviewVideo.value) {
    gifPreviewVideo.value.currentTime = gifOptions.startSeconds
  }
}

function syncSelectedGifVideo() {
  if (selectedGifVideo.value && videoFiles.value.includes(selectedGifVideo.value)) {
    return
  }

  selectedGifVideo.value = videoFiles.value[0] ?? ''
}

async function tauriInvoke<T>(command: string, payload?: Record<string, unknown>) {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(command, payload)
}

async function updateSelectedGifVideoSrc() {
  if (!selectedGifVideo.value) {
    selectedGifVideoSrc.value = ''
    selectedGifVideoDuration.value = 0
    gifPreviewError.value = ''
    return
  }

  selectedGifVideoSrc.value = await tauriInvoke<string>('get_media_preview_url', {
    path: selectedGifVideo.value
  })
  gifPreviewError.value = ''
}

async function pickFiles() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selection = await open({
    multiple: true,
    directory: false,
    filters: [
      {
        name: 'Media',
        extensions: ['mp4', 'mov', 'mkv', 'avi', 'webm', 'png', 'jpg', 'jpeg', 'webp', 'bmp', 'tiff', 'mp3', 'wav', 'aac', 'm4a', 'flac', 'opus', 'ogg']
      }
    ]
  })

  if (!selection) {
    return
  }

  const nextPaths = Array.isArray(selection) ? selection : [selection]
  files.value = Array.from(new Set([...files.value, ...nextPaths]))
  syncSelectedGifVideo()
  await updateSelectedGifVideoSrc()
}

async function pickOutputDir() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selection = await open({
    directory: true,
    multiple: false
  })

  if (typeof selection === 'string') {
    outputDir.value = selection
  }
}

function removeFile(path: string) {
  files.value = files.value.filter(item => item !== path)
  gifSegments.value = gifSegments.value.filter(segment => segment.inputPath !== path)
  syncSelectedGifVideo()
}

function clearQueue() {
  files.value = []
  results.value = []
  queueProgress.value = {}
  gifSegments.value = []
  activityQueue.value = []
  selectedGifVideo.value = ''
  selectedGifVideoSrc.value = ''
  selectedGifVideoDuration.value = 0
}

function clearCurrentRunState() {
  results.value = []
  queueProgress.value = {}
  cancelPending.value = false
}

function updateQueueProgress(event: BatchProgressEvent) {
  queueProgress.value = {
    ...queueProgress.value,
    [event.jobId]: {
      jobId: event.jobId,
      label: event.label,
      mediaKind: event.mediaKind,
      operation: event.operation,
      status: event.status,
      progressPercent: Math.round(event.progressPercent ?? 0),
      outputPath: event.outputPath,
      message: event.message,
      speed: event.speed
    }
  }
}

function isTerminalStatus(status: string | undefined) {
  return ['completed', 'failed', 'skipped', 'cancelled'].includes(status ?? '')
}

function statusColor(status: string | undefined) {
  if (status === 'completed') {
    return 'success'
  }

  if (status === 'failed') {
    return 'error'
  }

  if (status === 'cancelled' || status === 'skipped') {
    return 'warning'
  }

  return 'primary'
}

function formatUpdateDate(value: string | null) {
  if (!value) {
    return ''
  }

  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return value
  }

  return date.toLocaleDateString()
}

function showUpdateToast(status: AppUpdateStatus, failed = false) {
  const toastPayload = failed
    ? {
        title: 'Update check failed',
        description: status.message,
        icon: 'i-lucide-circle-alert',
        color: 'error' as const,
        duration: 7000
      }
    : status.updateInstalled
      ? {
          title: 'Update installed',
          description: status.message,
          icon: 'i-lucide-check-circle-2',
          color: 'success' as const,
          duration: 9000
        }
      : status.updateReady
        ? {
            title: `Version ${status.availableVersion} is available`,
            description: status.notes || status.message,
            icon: 'i-lucide-download',
            color: 'warning' as const,
            duration: 12000,
            actions: [
              {
                label: 'Install',
                color: 'success' as const,
                onClick: () => {
                  void installUpdate()
                }
              }
            ]
          }
        : {
            title: status.configured ? 'xcompressor is up to date' : 'Updater not configured',
            description: status.message,
            icon: status.configured ? 'i-lucide-badge-check' : 'i-lucide-info',
            color: status.configured ? 'success' as const : 'neutral' as const,
            duration: status.configured ? 5000 : 7000
          }

  if (updateToastId.value) {
    toast.update(updateToastId.value, toastPayload)
    return
  }

  updateToastId.value = toast.add(toastPayload).id
}

async function loadBootstrap() {
  bootstrap.value = await tauriInvoke<BootstrapData>('get_app_bootstrap')

  const videoTargets = targetFormats('video')
  const imageTargets = targetFormats('image')
  const audioTargets = targetFormats('audio')

  videoFormat.value = videoTargets[0] || 'mp4'
  imageFormat.value = imageTargets[0] || 'webp'
  audioFormat.value = audioTargets[0] || 'mp3'
}

async function checkForUpdates() {
  updateLoading.value = true

  try {
    updateStatus.value = await tauriInvoke<AppUpdateStatus>('check_for_app_update')
    showUpdateToast(updateStatus.value)
  } catch (error) {
    updateStatus.value = {
      configured: false,
      currentVersion: bootstrap.value?.version || 'unknown',
      availableVersion: null,
      notes: null,
      pubDate: null,
      updateReady: false,
      updateInstalled: false,
      message: String(error)
    }
    showUpdateToast(updateStatus.value, true)
  } finally {
    updateLoading.value = false
  }
}

async function installUpdate() {
  if (updateInstalling.value) {
    return
  }

  updateInstalling.value = true

  try {
    updateStatus.value = await tauriInvoke<AppUpdateStatus>('install_app_update')
    showUpdateToast(updateStatus.value)
  } catch (error) {
    updateStatus.value = {
      configured: updateStatus.value?.configured ?? false,
      currentVersion: updateStatus.value?.currentVersion || bootstrap.value?.version || 'unknown',
      availableVersion: updateStatus.value?.availableVersion ?? null,
      notes: updateStatus.value?.notes ?? null,
      pubDate: updateStatus.value?.pubDate ?? null,
      updateReady: updateStatus.value?.updateReady ?? false,
      updateInstalled: false,
      message: String(error)
    }
    showUpdateToast(updateStatus.value, true)
  } finally {
    updateInstalling.value = false
  }
}

function buildResourcePayload() {
  return {
    inputPaths: files.value,
    mode: mode.value,
    maxParallelJobs: maxParallelJobs.value,
    gifSegments: gifQueue.value,
    mixedJobs: activityQueue.value
  }
}

async function refreshResourcePlan() {
  resourcePlanLoading.value = true

  try {
    resourcePlan.value = await tauriInvoke<ResourcePlan>('analyze_resource_plan', {
      request: buildResourcePayload()
    })
  } catch {
    resourcePlan.value = null
  } finally {
    resourcePlanLoading.value = false
  }
}

async function refreshLiveSystemMetrics() {
  try {
    liveSystemMetrics.value = await tauriInvoke<LiveSystemMetrics>('get_live_system_metrics')
  } catch {
    liveSystemMetrics.value = null
  }
}

function startLiveMetricsPolling() {
  if (liveMetricsInterval) {
    clearInterval(liveMetricsInterval)
  }

  liveMetricsInterval = setInterval(() => {
    void refreshLiveSystemMetrics()
  }, 1500)
}

function stopLiveMetricsPolling() {
  if (liveMetricsInterval) {
    clearInterval(liveMetricsInterval)
    liveMetricsInterval = null
  }
}

function buildGifSegmentLabel(inputPath: string, index: number) {
  return `${basename(inputPath)} clip ${index + 1}`
}

function buildGifSegmentSuffix(index: number) {
  return `gif-${String(index + 1).padStart(2, '0')}`
}

function addGifSegment() {
  if (!selectedGifVideo.value) {
    return
  }

  clampGifRange()

  const segmentIndex
    = gifSegments.value.filter(segment => segment.inputPath === selectedGifVideo.value).length

  gifSegments.value = [
    ...gifSegments.value,
    {
      jobId: `${selectedGifVideo.value}::gif::${segmentIndex}::${gifOptions.startSeconds}::${gifOptions.durationSeconds}`,
      inputPath: selectedGifVideo.value,
      outputSuffix: buildGifSegmentSuffix(segmentIndex),
      label: buildGifSegmentLabel(selectedGifVideo.value, segmentIndex),
      startSeconds: gifOptions.startSeconds,
      durationSeconds: gifOptions.durationSeconds,
      fps: gifOptions.fps,
      width: gifOptions.width
    }
  ]
}

function removeGifSegment(jobId: string) {
  gifSegments.value = gifSegments.value.filter(segment => segment.jobId !== jobId)
}

function makeActivityJobBase(inputPath: string, nextMode: string) {
  return {
    inputPath,
    outputDir: outputDir.value,
    mode: nextMode,
    presetId: presetId.value,
    videoFormat: videoFormat.value,
    imageFormat: imageFormat.value,
    audioFormat: audioFormat.value,
    overwrite: true
  }
}

function addCurrentActivity() {
  if (!outputDir.value) {
    return
  }

  if (mode.value === 'gif') {
    const nextJobs = gifQueue.value.map(segment => ({
      ...makeActivityJobBase(segment.inputPath, 'gif'),
      jobId: `mixed::${segment.jobId}`,
      label: segment.label,
      resizeLongEdge: null,
      gif: {
        startSeconds: segment.startSeconds,
        durationSeconds: segment.durationSeconds,
        fps: segment.fps,
        width: segment.width
      },
      outputSuffix: segment.outputSuffix
    } satisfies MixedJob))

    activityQueue.value = [
      ...activityQueue.value,
      ...nextJobs.filter(job => !activityQueue.value.some(existing => existing.jobId === job.jobId))
    ]
    return
  }

  const nextJobs = files.value.map(path => ({
    ...makeActivityJobBase(path, mode.value),
    jobId: `mixed::${mode.value}::${path}`,
    label: `${basename(path)} • ${mode.value}`,
    resizeLongEdge: resizeLongEdge.value,
    gif: null,
    outputSuffix: null
  } satisfies MixedJob))

  activityQueue.value = [
    ...activityQueue.value,
    ...nextJobs.filter(job => !activityQueue.value.some(existing => existing.jobId === job.jobId))
  ]
}

function removeActivityJob(jobId: string) {
  activityQueue.value = activityQueue.value.filter(job => job.jobId !== jobId)
}

function clearActivityQueue() {
  activityQueue.value = []
}

async function registerBatchListener() {
  const { listen } = await import('@tauri-apps/api/event')
  unlistenBatchProgress = await listen<BatchProgressEvent>('batch-progress', ({ payload }) => {
    updateQueueProgress(payload)
  })
}

async function runBatch() {
  clearCurrentRunState()
  currentRunId.value = `run-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
  processing.value = true
  cancelPending.value = false
  queueProgress.value = activityQueue.value.length
    ? Object.fromEntries(
        activityQueue.value.map(job => [job.jobId, {
          jobId: job.jobId,
          label: job.label,
          mediaKind: detectKind(job.inputPath),
          operation: job.mode,
          status: 'queued',
          progressPercent: 0,
          outputPath: null,
          message: `Queued ${job.mode} job.`,
          speed: null
        } satisfies QueueProgress])
      )
    : mode.value === 'gif'
      ? Object.fromEntries(
          gifQueue.value.map(segment => [segment.jobId, {
            jobId: segment.jobId,
            label: segment.label,
            mediaKind: detectKind(segment.inputPath),
            operation: mode.value,
            status: 'queued',
            progressPercent: 0,
            outputPath: null,
            message: 'Queued GIF clip for batch processing.',
            speed: null
          } satisfies QueueProgress])
        )
      : Object.fromEntries(
          files.value.map(path => [batchJobId(path, mode.value), {
            jobId: batchJobId(path, mode.value),
            label: basename(path),
            mediaKind: detectKind(path),
            operation: mode.value,
            status: 'queued',
            progressPercent: 0,
            outputPath: null,
            message: 'Queued for batch processing.',
            speed: null
          } satisfies QueueProgress])
        )

  try {
    const response = await tauriInvoke<{ results: BatchJobResult[] }>('run_batch_jobs', {
      request: {
        runId: currentRunId.value,
        inputPaths: files.value,
        outputDir: outputDir.value,
        mode: mode.value,
        presetId: presetId.value,
        videoFormat: videoFormat.value,
        imageFormat: imageFormat.value,
        audioFormat: audioFormat.value,
        resizeLongEdge: mode.value === 'gif' ? null : resizeLongEdge.value,
        maxParallelJobs: maxParallelJobs.value,
        gif: {
          startSeconds: gifOptions.startSeconds,
          durationSeconds: gifOptions.durationSeconds,
          fps: gifOptions.fps,
          width: gifOptions.width
        },
        gifSegments: gifQueue.value,
        mixedJobs: activityQueue.value,
        overwrite: true
      }
    })

    results.value = response.results
  } catch (error) {
    results.value = [{
      jobId: 'batch-error',
      label: 'Batch error',
      inputPath: 'batch',
      mediaKind: 'system',
      operation: mode.value,
      outputPath: null,
      success: false,
      skipped: false,
      cancelled: false,
      ffmpegArgs: [],
      message: String(error)
    }]
  } finally {
    processing.value = false
    cancelPending.value = false
    currentRunId.value = ''
  }
}

async function cancelBatch() {
  if (!processing.value || !currentRunId.value || cancelPending.value) {
    return
  }

  cancelPending.value = true

  try {
    await tauriInvoke('cancel_batch_run', { runId: currentRunId.value })
  } catch {
    cancelPending.value = false
  }
}

function enableSequentialMode() {
  maxParallelJobs.value = 1
}

async function openSelectedGifVideoInSystemPlayer() {
  if (!selectedGifVideo.value) {
    return
  }

  await tauriInvoke('open_media_in_system_player', { path: selectedGifVideo.value })
}

onMounted(async () => {
  await loadBootstrap()
  await checkForUpdates()
  syncSelectedGifVideo()
  await updateSelectedGifVideoSrc()
  await registerBatchListener()
  await refreshResourcePlan()
  await refreshLiveSystemMetrics()
  startLiveMetricsPolling()
})

onBeforeUnmount(() => {
  unlistenBatchProgress?.()
  stopLiveMetricsPolling()
})

watch(mode, async () => {
  syncSelectedGifVideo()
  await updateSelectedGifVideoSrc()
  await refreshResourcePlan()
})

watch(videoFiles, async () => {
  syncSelectedGifVideo()
  await updateSelectedGifVideoSrc()
  await refreshResourcePlan()
})

watch(selectedGifVideoDuration, () => {
  clampGifRange()
})

watch([files, gifSegments, activityQueue, maxParallelJobs, outputDir, presetId, resizeLongEdge, videoFormat, imageFormat, audioFormat], async () => {
  await refreshResourcePlan()
}, { deep: true })

function onGifVideoLoaded(event: Event) {
  const target = event.target as HTMLVideoElement
  selectedGifVideoDuration.value = Number.isFinite(target.duration) ? target.duration : 0
  gifPreviewVideo.value = target
  gifPreviewError.value = ''
  clampGifRange()
}

function setGifStartFromPreview(event: Event) {
  const target = event.target as HTMLVideoElement
  setGifStart(Number(target.currentTime.toFixed(2)))
}

function onGifVideoError() {
  gifPreviewVideo.value = null
  gifPreviewError.value = 'Embedded preview failed in the desktop webview for this file or codec. Open it in the system player instead.'
}
</script>

<template>
  <div class="min-h-screen text-stone-100">
    <div class="mx-auto max-w-[1700px] px-4 py-4 lg:px-6">
      <div class="grid min-h-[calc(100vh-2rem)] gap-4 xl:grid-cols-[300px_minmax(0,1.35fr)_420px]">
        <aside class="grid min-h-0 gap-4">
          <UCard :ui="{ root: 'h-full border border-white/10 bg-stone-950/85 ring-0' }">
            <template #header>
              <div class="space-y-3">
                <div class="flex flex-wrap items-center gap-2">
                  <UBadge color="primary" variant="soft" label="Workspace overview" />
                  <UBadge v-if="bootstrap?.version" color="neutral" variant="soft" :label="`v${bootstrap.version}`" />
                </div>
                <div>
                  <h1 class="text-3xl font-semibold tracking-tight text-white">
                    xcompressor
                  </h1>
                  <p class="mt-2 text-sm leading-6 text-stone-300">
                    Multimedia compression, conversion, and GIF jobs arranged like a desktop workspace instead of a landing page.
                  </p>
                </div>
              </div>
            </template>

            <div class="space-y-4">
              <div class="rounded-2xl border border-amber-500/20 bg-amber-500/8 p-4">
                <p class="text-xs font-semibold uppercase tracking-[0.25em] text-amber-300">
                  Active preset
                </p>
                <p class="mt-2 text-xl font-medium text-white">
                  {{ activePreset?.label || 'Balanced' }}
                </p>
                <p class="mt-2 text-sm leading-6 text-stone-300">
                  {{ activePreset?.description }}
                </p>
                <div class="mt-3 grid grid-cols-2 gap-3 text-sm">
                  <div>
                    <p class="text-stone-500">
                      Quality
                    </p>
                    <p class="font-medium text-stone-100">
                      {{ activePreset?.qualityRange }}
                    </p>
                  </div>
                  <div>
                    <p class="text-stone-500">
                      Size delta
                    </p>
                    <p class="font-medium text-stone-100">
                      {{ activePreset?.sizeReductionRange }}
                    </p>
                  </div>
                </div>
              </div>

              <div class="space-y-3">
                <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
                  Capabilities
                </p>
                <div
                  v-for="capability in bootstrap?.mediaCapabilities || []"
                  :key="capability.kind"
                  class="rounded-2xl border border-white/8 bg-white/5 p-4"
                >
                  <div class="flex items-center justify-between gap-3">
                    <h2 class="text-sm font-medium capitalize text-white">
                      {{ capability.kind }}
                    </h2>
                    <UBadge color="neutral" variant="soft" :label="capability.conversions.length + ' targets'" />
                  </div>
                  <p class="mt-2 text-xs leading-6 text-stone-400">
                    {{ capability.compressionModes.join(' • ') }}
                  </p>
                </div>
              </div>

              <div class="space-y-3">
                <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
                  GIF workflow
                </p>
                <ol class="space-y-2">
                  <li
                    v-for="(step, index) in bootstrap?.gifWorkflow || []"
                    :key="step"
                    class="flex gap-3 rounded-2xl border border-white/8 bg-white/5 p-3"
                  >
                    <div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-amber-500/15 text-xs font-semibold text-amber-300">
                      {{ index + 1 }}
                    </div>
                    <p class="text-xs leading-6 text-stone-300">
                      {{ step }}
                    </p>
                  </li>
                </ol>
              </div>
            </div>
          </UCard>
        </aside>

        <main class="grid min-h-0 gap-4">
          <UCard :ui="{ root: 'border border-white/10 bg-stone-950/85 ring-0' }">
            <template #header>
              <div class="flex items-center justify-between gap-4">
                <div>
                  <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
                    Configure
                  </p>
                  <h2 class="mt-2 text-2xl font-semibold text-white">
                    Files and operations
                  </h2>
                </div>
                <UBadge color="primary" variant="soft" :label="files.length + ' source files'" />
              </div>
            </template>

            <div class="space-y-5">
              <div class="flex flex-wrap gap-3">
                <UButton icon="i-lucide-folder-plus" color="primary" size="lg" @click="pickFiles">
                  Add media
                </UButton>
                <UButton icon="i-lucide-folder-output" color="neutral" variant="soft" size="lg" @click="pickOutputDir">
                  Output directory
                </UButton>
                <UButton icon="i-lucide-trash-2" color="error" variant="ghost" size="lg" @click="clearQueue">
                  Clear all
                </UButton>
              </div>

              <UInput
                v-model="outputDir"
                icon="i-lucide-folder-open"
                size="xl"
                placeholder="Choose an output directory"
              />

              <div class="grid gap-4 lg:grid-cols-2">
                <UFormField label="Mode">
                  <USelect v-model="mode" :items="modeOptions" option-attribute="label" value-attribute="value" />
                </UFormField>

                <UFormField label="Preset">
                  <USelect
                    v-model="presetId"
                    :items="(bootstrap?.presets || []).map(preset => ({ label: preset.label, value: preset.id }))"
                    option-attribute="label"
                    value-attribute="value"
                  />
                </UFormField>

                <UFormField label="Video target">
                  <USelect v-model="videoFormat" :items="targetFormats('video')" />
                </UFormField>

                <UFormField label="Image target">
                  <USelect v-model="imageFormat" :items="targetFormats('image')" />
                </UFormField>

                <UFormField label="Audio target">
                  <USelect v-model="audioFormat" :items="targetFormats('audio')" />
                </UFormField>

                <UFormField label="Parallel jobs">
                  <UInputNumber v-model="maxParallelJobs" :min="1" :max="8" />
                </UFormField>

                <UFormField v-if="mode !== 'gif'" label="Resize long edge">
                  <UInputNumber v-model="resizeLongEdge" :min="320" :max="4096" :step="10" />
                </UFormField>
              </div>

              <div class="grid gap-3 md:grid-cols-2">
                <UButton
                  block
                  size="xl"
                  color="neutral"
                  variant="soft"
                  icon="i-lucide-plus"
                  :disabled="!outputDir || (mode === 'gif' ? !gifQueue.length : !files.length)"
                  @click="addCurrentActivity"
                >
                  Add current activity
                </UButton>
                <UButton
                  block
                  size="xl"
                  color="primary"
                  icon="i-lucide-play"
                  :loading="processing"
                  :disabled="!canRun"
                  @click="runBatch"
                >
                  {{ processing ? 'Processing batch...' : (activityQueue.length ? 'Run mixed activity batch' : (mode === 'gif' ? 'Generate GIF batch' : 'Run batch')) }}
                </UButton>
              </div>

              <UButton
                v-if="processing"
                block
                size="lg"
                color="warning"
                variant="soft"
                icon="i-lucide-square"
                :loading="cancelPending"
                @click="cancelBatch"
              >
                {{ cancelPending ? 'Stopping batch...' : 'Cancel running batch' }}
              </UButton>

              <p v-if="mode === 'gif' && !gifQueue.length" class="text-sm leading-6 text-amber-300">
                Add at least one GIF clip before running export.
              </p>
              <p v-if="activityQueue.length" class="text-sm leading-6 text-sky-300">
                Saved activities override the current editor when you run the batch.
              </p>
            </div>
          </UCard>

          <UCard v-if="mode === 'gif'" :ui="{ root: 'border border-amber-500/20 bg-stone-950/85 ring-0' }">
            <template #header>
              <div class="flex items-center justify-between gap-4">
                <div>
                  <p class="text-xs font-semibold uppercase tracking-[0.25em] text-amber-300">
                    GIF editor
                  </p>
                  <h2 class="mt-2 text-2xl font-semibold text-white">
                    Preview and clip ranges
                  </h2>
                </div>
                <UBadge color="warning" variant="soft" :label="gifQueue.length + ' clips queued'" />
              </div>
            </template>

            <div class="grid gap-4 xl:grid-cols-[1.2fr_0.8fr]">
              <div class="min-w-0 space-y-4 overflow-hidden rounded-2xl border border-white/10 bg-black/20 p-4">
                <UFormField label="Preview source" description="Choose the video you want to clip from.">
                  <USelect
                    v-model="selectedGifVideo"
                    :items="videoFileOptions"
                    option-attribute="label"
                    value-attribute="value"
                    :ui="{ base: 'min-w-0 w-full', content: 'max-w-full' }"
                  />
                </UFormField>

                <div v-if="selectedGifVideoSrc" class="min-w-0 space-y-3">
                  <p class="truncate text-xs text-stone-500">
                    {{ basename(selectedGifVideo) }}
                  </p>
                  <video
                    ref="gifPreviewVideo"
                    class="block aspect-video max-w-full rounded-2xl border border-white/10 bg-black object-contain"
                    :src="selectedGifVideoSrc"
                    controls
                    preload="metadata"
                    @loadedmetadata="onGifVideoLoaded"
                    @pause="setGifStartFromPreview"
                    @error="onGifVideoError"
                  />
                  <div v-if="gifPreviewError" class="rounded-2xl border border-amber-500/20 bg-amber-500/8 p-4">
                    <p class="text-sm leading-6 text-amber-200">
                      {{ gifPreviewError }}
                    </p>
                    <UButton
                      class="mt-3"
                      color="warning"
                      variant="soft"
                      icon="i-lucide-external-link"
                      @click="openSelectedGifVideoInSystemPlayer"
                    >
                      Open in system player
                    </UButton>
                  </div>
                  <div class="flex flex-wrap gap-3">
                    <UButton
                      color="neutral"
                      variant="soft"
                      icon="i-lucide-scissors"
                      :disabled="Boolean(gifPreviewError) || !gifPreviewVideo"
                      @click="setGifStartFromPreview($event)"
                    >
                      Use paused time as start
                    </UButton>
                    <UButton
                      color="neutral"
                      variant="soft"
                      icon="i-lucide-skip-back"
                      :disabled="Boolean(gifPreviewError) || !gifPreviewVideo"
                      @click="syncPreviewToGifStart"
                    >
                      Jump to clip start
                    </UButton>
                    <UButton color="neutral" variant="soft" icon="i-lucide-monitor-play" @click="openSelectedGifVideoInSystemPlayer">
                      Open externally
                    </UButton>
                    <p class="text-xs leading-6 text-stone-400">
                      Video length: {{ selectedGifVideoDuration ? `${selectedGifVideoDuration.toFixed(1)}s` : 'loading...' }}
                    </p>
                  </div>

                  <div class="rounded-2xl border border-white/10 bg-white/5 p-4">
                    <div class="mb-3 flex items-center justify-between gap-3">
                      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-stone-400">
                        Clip range
                      </p>
                      <p class="text-sm font-medium text-white">
                        {{ gifClipRangeLabel }}
                      </p>
                    </div>
                    <div class="space-y-4">
                      <div>
                        <div class="mb-2 flex items-center justify-between text-xs text-stone-400">
                          <span>Start</span>
                          <span>{{ gifOptions.startSeconds.toFixed(1) }}s</span>
                        </div>
                        <input
                          :value="gifOptions.startSeconds"
                          class="h-2 w-full cursor-pointer appearance-none rounded-full bg-stone-700 accent-amber-400"
                          type="range"
                          min="0"
                          :max="Math.max(selectedGifVideoDuration - 0.5, 0)"
                          step="0.1"
                          @input="setGifStart(Number(($event.target as HTMLInputElement).value))"
                        >
                      </div>
                      <div>
                        <div class="mb-2 flex items-center justify-between text-xs text-stone-400">
                          <span>End</span>
                          <span>{{ gifEndSeconds.toFixed(1) }}s</span>
                        </div>
                        <input
                          :value="gifEndSeconds"
                          class="h-2 w-full cursor-pointer appearance-none rounded-full bg-stone-700 accent-sky-400"
                          type="range"
                          :min="Math.min(gifOptions.startSeconds + 0.5, selectedGifVideoDuration || gifOptions.startSeconds + 0.5)"
                          :max="selectedGifVideoDuration || gifOptions.startSeconds + gifOptions.durationSeconds"
                          step="0.1"
                          @input="setGifEnd(Number(($event.target as HTMLInputElement).value))"
                        >
                      </div>
                    </div>
                  </div>
                </div>

                <UEmptyState
                  v-else
                  icon="i-lucide-film"
                  title="No video selected for preview"
                  description="Add at least one video file to start building GIF clips."
                />
              </div>

              <div class="space-y-4">
                <div class="rounded-2xl border border-white/10 bg-black/20 p-4">
                  <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-1">
                    <UFormField label="Start second" description="Where the clip begins inside the source video.">
                      <UInputNumber v-model="gifOptions.startSeconds" :min="0" :step="0.5" />
                    </UFormField>
                    <UFormField label="End second" description="Where the clip stops inside the source video.">
                      <UInputNumber v-model="gifEndSeconds" :min="0.5" :step="0.5" />
                    </UFormField>
                    <UFormField label="GIF FPS" description="Higher FPS is smoother but creates a larger file.">
                      <UInputNumber v-model="gifOptions.fps" :min="1" :max="30" />
                    </UFormField>
                    <UFormField label="GIF width" description="Output width in pixels. Smaller width exports faster.">
                      <UInputNumber v-model="gifOptions.width" :min="160" :max="1280" :step="20" />
                    </UFormField>
                  </div>

                  <UButton
                    block
                    class="mt-4"
                    color="primary"
                    icon="i-lucide-plus"
                    :disabled="!selectedGifVideo"
                    @click="addGifSegment"
                  >
                    Add clip to GIF queue
                  </UButton>
                </div>

                <div class="rounded-2xl border border-white/10 bg-black/20 p-4">
                  <p class="text-xs font-semibold uppercase tracking-[0.2em] text-stone-400">
                    GIF summary
                  </p>
                  <div class="mt-3 grid gap-3 sm:grid-cols-3 xl:grid-cols-1">
                    <div>
                      <p class="text-xs text-stone-500">
                        Clip range
                      </p>
                      <p class="text-sm font-medium text-white">
                        {{ gifClipRangeLabel }}
                      </p>
                    </div>
                    <div>
                      <p class="text-xs text-stone-500">
                        Motion
                      </p>
                      <p class="text-sm font-medium text-white">
                        {{ gifOptions.fps }} fps
                      </p>
                    </div>
                    <div>
                      <p class="text-xs text-stone-500">
                        Output
                      </p>
                      <p class="text-sm font-medium text-white">
                        {{ gifOptions.width }}px wide `.gif`
                      </p>
                    </div>
                  </div>
                  <p class="mt-3 text-sm leading-6 text-stone-300">
                    Eligible video files: {{ videoFiles.length }}. Queued GIF clips: {{ gifQueue.length }}<span v-if="nonVideoFiles.length">. Non-video files stay in the media queue but cannot create GIF clips.</span>
                  </p>
                </div>
              </div>
            </div>
          </UCard>

          <div class="grid min-h-0 gap-4 lg:grid-cols-2">
            <UCard :ui="{ root: 'border border-white/10 bg-stone-950/85 ring-0' }">
              <template #header>
                <div class="flex items-center justify-between gap-4">
                  <div>
                    <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
                      Activity batch
                    </p>
                    <h2 class="mt-2 text-xl font-semibold text-white">
                      Saved operations
                    </h2>
                  </div>
                  <div class="flex items-center gap-3">
                    <UBadge color="primary" variant="soft" :label="activityQueue.length + ' queued'" />
                    <UButton
                      icon="i-lucide-trash-2"
                      color="error"
                      variant="ghost"
                      :disabled="!activityQueue.length"
                      @click="clearActivityQueue"
                    >
                      Clear
                    </UButton>
                  </div>
                </div>
              </template>

              <div v-if="activityQueue.length" class="max-h-[32rem] space-y-3 overflow-y-auto pr-1">
                <div
                  v-for="job in activityQueue"
                  :key="job.jobId"
                  class="flex flex-col gap-3 rounded-2xl border border-white/8 bg-white/5 p-4"
                >
                  <div class="flex flex-wrap items-center gap-2">
                    <p class="truncate text-sm font-medium text-white">
                      {{ job.label }}
                    </p>
                    <UBadge color="neutral" variant="soft" :label="job.mode" />
                    <UBadge color="neutral" variant="soft" :label="detectKind(job.inputPath)" />
                    <UBadge
                      v-if="mixedJobProgress(job)"
                      :color="statusColor(mixedJobProgress(job)?.status)"
                      variant="soft"
                      :label="mixedJobProgress(job)?.status || 'queued'"
                    />
                  </div>
                  <p class="truncate text-xs text-stone-500">
                    {{ job.inputPath }}
                  </p>
                  <p class="text-xs text-stone-400">
                    {{ describeActivity(job) }}
                  </p>
                  <div v-if="mixedJobProgress(job)" class="space-y-2">
                    <div class="h-2 overflow-hidden rounded-full bg-white/8">
                      <div
                        class="h-full rounded-full bg-gradient-to-r from-amber-400 via-orange-400 to-sky-400 transition-[width] duration-300"
                        :style="{ width: `${mixedJobProgress(job)?.progressPercent ?? 0}%` }"
                      />
                    </div>
                    <div class="flex flex-wrap items-center justify-between gap-2 text-xs text-stone-400">
                      <p>{{ mixedJobProgress(job)?.message }}</p>
                      <p>{{ mixedJobProgress(job)?.progressPercent ?? 0 }}%</p>
                    </div>
                  </div>
                  <div class="flex justify-end">
                    <UButton
                      icon="i-lucide-x"
                      color="neutral"
                      variant="ghost"
                      @click="removeActivityJob(job.jobId)"
                    >
                      Remove
                    </UButton>
                  </div>
                </div>
              </div>

              <UEmptyState
                v-else
                icon="i-lucide-layers-3"
                title="No mixed activity batch yet"
                description="Build a compress, convert, or GIF configuration above, then add it here as a saved activity."
              />
            </UCard>

            <UCard :ui="{ root: 'border border-white/10 bg-stone-950/85 ring-0' }">
              <template #header>
                <div class="flex items-center justify-between gap-4">
                  <div>
                    <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
                      Source queue
                    </p>
                    <h2 class="mt-2 text-xl font-semibold text-white">
                      Selected media
                    </h2>
                  </div>
                  <p class="max-w-xs text-right text-sm text-stone-400">
                    <span v-if="activityQueue.length">
                      These files feed the editor. The saved activity list is what runs.
                    </span>
                    <span v-else-if="mode === 'gif'">
                      GIF export is driven by the queued clips below.
                    </span>
                    <span v-else>
                      Mixed video, image, and audio files are supported in one run.
                    </span>
                  </p>
                </div>
              </template>

              <div v-if="mode === 'gif' ? gifQueue.length : files.length" class="max-h-[32rem] space-y-3 overflow-y-auto pr-1">
                <div
                  v-for="item in (mode === 'gif' ? gifQueue : files)"
                  :key="typeof item === 'string' ? item : item.jobId"
                  class="flex flex-col gap-3 rounded-2xl border border-white/8 bg-white/5 p-4"
                >
                  <div class="flex flex-wrap items-center gap-2">
                    <p class="truncate text-sm font-medium text-white">
                      {{ typeof item === 'string' ? basename(item) : item.label }}
                    </p>
                    <UBadge color="neutral" variant="soft" :label="detectKind(typeof item === 'string' ? item : item.inputPath)" />
                    <UBadge
                      v-if="mode === 'gif' && typeof item === 'string' && detectKind(item) !== 'video'"
                      color="warning"
                      variant="soft"
                      label="skipped in gif mode"
                    />
                    <UBadge
                      v-if="queueItemProgress(item)"
                      :color="statusColor(queueItemProgress(item)?.status)"
                      variant="soft"
                      :label="queueItemProgress(item)?.status || 'queued'"
                    />
                  </div>
                  <p class="truncate text-xs text-stone-500">
                    {{ typeof item === 'string' ? item : item.inputPath }}
                  </p>
                  <p v-if="mode === 'gif' && typeof item !== 'string'" class="text-xs text-stone-400">
                    {{ item.startSeconds.toFixed(1) }}s -> {{ (item.startSeconds + item.durationSeconds).toFixed(1) }}s • {{ item.fps }} fps • {{ item.width }}px
                  </p>
                  <div v-if="queueItemProgress(item)" class="space-y-2">
                    <div class="h-2 overflow-hidden rounded-full bg-white/8">
                      <div
                        class="h-full rounded-full bg-gradient-to-r from-amber-400 via-orange-400 to-sky-400 transition-[width] duration-300"
                        :style="{ width: `${queueItemProgress(item)?.progressPercent ?? 0}%` }"
                      />
                    </div>
                    <div class="flex flex-wrap items-center justify-between gap-2 text-xs text-stone-400">
                      <p>{{ queueItemProgress(item)?.message }}</p>
                      <p>{{ queueItemProgress(item)?.progressPercent ?? 0 }}%</p>
                    </div>
                    <p v-if="queueItemProgress(item)?.speed" class="text-xs text-stone-500">
                      Speed: {{ queueItemProgress(item)?.speed }}
                    </p>
                  </div>
                  <p
                    v-else-if="mode === 'gif' && typeof item === 'string' && detectKind(item) !== 'video'"
                    class="text-xs text-amber-300"
                  >
                    This file stays in the queue, but GIF export only runs on video inputs.
                  </p>
                  <div class="flex justify-end">
                    <UButton
                      icon="i-lucide-x"
                      color="neutral"
                      variant="ghost"
                      @click="typeof item === 'string' ? removeFile(item) : removeGifSegment(item.jobId)"
                    >
                      {{ mode === 'gif' && typeof item !== 'string' ? 'Remove clip' : 'Remove' }}
                    </UButton>
                  </div>
                </div>
              </div>

              <UEmptyState
                v-else
                icon="i-lucide-clapperboard"
                :title="mode === 'gif' ? 'No GIF clips queued' : 'No media queued'"
                :description="mode === 'gif' ? 'Preview a video and add one or more clip ranges to the GIF queue.' : 'Use Add media to select videos, images, and audio files from disk.'"
              />
            </UCard>
          </div>
        </main>

        <aside class="grid min-h-0 gap-4">
          <UCard :ui="{ root: 'border border-white/10 bg-stone-950/85 ring-0' }">
            <template #header>
              <div class="flex items-center justify-between gap-4">
                <div>
                  <p class="text-xs font-semibold uppercase tracking-[0.25em] text-emerald-300">
                    App updates
                  </p>
                  <p class="mt-2 text-sm leading-6 text-stone-300">
                    {{ updateLoading ? 'Checking GitHub release channel...' : (updateStatus?.message || 'Update status unavailable.') }}
                  </p>
                </div>
                <UBadge
                  v-if="updateStatus"
                  :color="updateStatus.updateReady ? 'warning' : (updateStatus.configured ? 'success' : 'neutral')"
                  variant="soft"
                  :label="updateStatus.updateReady ? 'update ready' : (updateStatus.configured ? 'current' : 'not configured')"
                />
              </div>
            </template>

            <div class="space-y-3">
              <div class="grid grid-cols-2 gap-3">
                <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
                  <p class="text-xs text-stone-500">
                    Installed
                  </p>
                  <p class="mt-1 text-lg font-semibold text-white">
                    {{ updateStatus?.currentVersion || bootstrap?.version || 'n/a' }}
                  </p>
                </div>
                <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
                  <p class="text-xs text-stone-500">
                    Available
                  </p>
                  <p class="mt-1 text-lg font-semibold text-white">
                    {{ updateStatus?.availableVersion || 'latest' }}
                  </p>
                </div>
              </div>

              <p v-if="updateStatus?.pubDate" class="text-xs leading-6 text-stone-400">
                Release date: {{ formatUpdateDate(updateStatus.pubDate) }}
              </p>
              <p v-if="updateStatus?.notes" class="text-sm leading-6 text-stone-300">
                {{ updateStatus.notes }}
              </p>

              <div class="flex flex-wrap gap-3">
                <UButton
                  color="neutral"
                  variant="soft"
                  icon="i-lucide-refresh-ccw"
                  :loading="updateLoading"
                  @click="checkForUpdates"
                >
                  Check now
                </UButton>
                <UButton
                  v-if="updateStatus?.updateReady"
                  color="success"
                  icon="i-lucide-download"
                  :loading="updateInstalling"
                  @click="installUpdate"
                >
                  Install update
                </UButton>
              </div>
            </div>
          </UCard>

          <UCard :ui="{ root: 'border border-white/10 bg-stone-950/85 ring-0' }">
            <template #header>
              <div>
                <p class="text-xs font-semibold uppercase tracking-[0.25em] text-sky-300">
                  Resource planner
                </p>
                <p class="mt-2 text-sm leading-6 text-stone-300">
                  {{ resourcePlanLoading ? 'Checking available resources...' : (resourcePlan?.summary || 'No jobs selected yet.') }}
                </p>
              </div>
            </template>

            <div class="space-y-3">
              <div class="grid grid-cols-2 gap-3">
                <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
                  <p class="text-xs text-stone-500">
                    Live CPU
                  </p>
                  <p class="mt-1 text-lg font-semibold text-white">
                    {{ liveSystemMetrics ? `${Math.round(liveSystemMetrics.cpuUsagePercent)}%` : 'n/a' }}
                  </p>
                </div>
                <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
                  <p class="text-xs text-stone-500">
                    Live RAM
                  </p>
                  <p class="mt-1 text-lg font-semibold text-white">
                    {{ liveSystemMetrics ? `${liveSystemMetrics.usedMemoryMb} MB` : 'n/a' }}
                  </p>
                </div>
                <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
                  <p class="text-xs text-stone-500">
                    Planned RAM
                  </p>
                  <p class="mt-1 text-lg font-semibold text-white">
                    {{ resourcePlan?.estimatedParallelMemoryMb ? `${resourcePlan.estimatedParallelMemoryMb} MB` : 'n/a' }}
                  </p>
                </div>
                <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
                  <p class="text-xs text-stone-500">
                    {{ etaCaption }}
                  </p>
                  <p class="mt-1 text-lg font-semibold text-white">
                    {{ estimatedMinutesLabel }}
                  </p>
                </div>
              </div>

              <div class="flex flex-wrap items-center gap-3">
                <UButton
                  v-if="processing"
                  color="warning"
                  variant="soft"
                  icon="i-lucide-square"
                  :loading="cancelPending"
                  @click="cancelBatch"
                >
                  {{ cancelPending ? 'Stopping...' : 'Cancel batch' }}
                </UButton>
                <UBadge
                  :color="resourcePlan?.canRunInParallel === false ? 'warning' : 'success'"
                  variant="soft"
                  :label="resourcePlan?.canRunInParallel === false ? 'parallel limited' : 'parallel OK'"
                />
                <p class="text-xs leading-6 text-stone-400">
                  Jobs {{ resourcePlan?.jobs.length ?? runQueueCount }} • requested {{ effectiveParallelJobs }} • safe {{ resourcePlan?.safeParallelJobs ?? 1 }}
                </p>
                <p class="text-xs leading-6 text-stone-400">
                  Available RAM {{ liveSystemMetrics ? `${liveSystemMetrics.availableMemoryMb} MB` : 'n/a' }}
                </p>
              </div>

              <UButton
                v-if="resourcePlan?.shouldUseSequential && effectiveParallelJobs > 1"
                block
                color="warning"
                variant="soft"
                icon="i-lucide-git-commit-horizontal"
                @click="enableSequentialMode"
              >
                Switch to sequential mode
              </UButton>

              <p v-if="resourcePlan?.shouldUseSequential && effectiveParallelJobs > 1" class="text-sm leading-6 text-amber-300">
                Parallel execution is blocked for the current estimate.
              </p>
            </div>
          </UCard>

          <UCard :ui="{ root: 'border border-white/10 bg-stone-950/85 ring-0' }">
            <template #header>
              <div class="flex items-center justify-between gap-4">
                <div>
                  <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
                    Run status
                  </p>
                  <h2 class="mt-2 text-2xl font-semibold text-white">
                    Batch monitor
                  </h2>
                </div>
                <p class="text-2xl font-semibold text-white">
                  {{ overallProgress }}%
                </p>
              </div>
            </template>

            <div class="space-y-4">
              <p class="text-sm leading-6 text-stone-300">
                <span v-if="activityQueue.length">
                  {{ completedJobs }}/{{ activityQueue.length || 0 }} mixed activity jobs finished.
                </span>
                <span v-else-if="mode === 'gif'">
                  {{ completedJobs }}/{{ gifQueue.length || 0 }} queued GIF clips finished.
                </span>
                <span v-else>
                  {{ completedJobs }}/{{ files.length || 0 }} jobs finished.
                </span>
                <span v-if="cancelPending" class="text-amber-300">
                  Cancellation requested. Active FFmpeg jobs are being stopped.
                </span>
              </p>

              <div class="h-3 overflow-hidden rounded-full bg-white/8">
                <div
                  class="h-full rounded-full bg-gradient-to-r from-amber-400 via-orange-400 to-sky-400 transition-[width] duration-300"
                  :style="{ width: `${overallProgress}%` }"
                />
              </div>
            </div>
          </UCard>

          <UCard :ui="{ root: 'h-full border border-white/10 bg-stone-950/85 ring-0' }">
            <template #header>
              <div class="flex items-center justify-between gap-4">
                <div>
                  <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
                    Batch output
                  </p>
                  <h2 class="mt-2 text-2xl font-semibold text-white">
                    Results
                  </h2>
                </div>
                <UBadge
                  v-if="results.length"
                  color="primary"
                  variant="soft"
                  :label="results.filter(result => result.success).length + ' successful'"
                />
              </div>
            </template>

            <div v-if="results.length" class="max-h-[calc(100vh-15rem)] space-y-3 overflow-y-auto pr-1">
              <div
                v-for="result in results"
                :key="result.jobId"
                class="rounded-2xl border p-3"
                :class="result.success ? 'border-emerald-500/30 bg-emerald-500/8' : (result.cancelled || result.skipped ? 'border-amber-500/30 bg-amber-500/8' : 'border-rose-500/30 bg-rose-500/8')"
              >
                <div class="flex flex-wrap items-center gap-2">
                  <UBadge
                    :color="result.success ? 'success' : (result.cancelled || result.skipped ? 'warning' : 'error')"
                    variant="soft"
                    :label="result.success ? 'success' : (result.cancelled ? 'cancelled' : (result.skipped ? 'skipped' : 'failed'))"
                  />
                  <UBadge color="neutral" variant="soft" :label="result.mediaKind" />
                  <p class="text-sm font-medium text-white">
                    {{ result.label || basename(result.inputPath) }}
                  </p>
                </div>

                <p class="mt-2 line-clamp-3 text-sm leading-6 text-stone-300">
                  {{ result.message }}
                </p>

                <p v-if="result.outputPath" class="mt-2 truncate text-xs text-stone-400">
                  Output: {{ basename(result.outputPath) }}
                </p>

                <details v-if="result.ffmpegArgs.length" class="mt-2">
                  <summary class="cursor-pointer text-xs font-medium uppercase tracking-[0.2em] text-stone-400">
                    ffmpeg args
                  </summary>
                  <pre class="mt-3 overflow-x-auto rounded-xl bg-black/40 p-3 text-xs text-stone-300">{{ result.ffmpegArgs.join(' ') }}</pre>
                </details>
              </div>
            </div>

            <UEmptyState
              v-else
              icon="i-lucide-badge-check"
              title="No jobs have run yet"
              description="Per-job results appear here while the workspace stays focused on setup in the center pane."
            />
          </UCard>
        </aside>
      </div>
    </div>
  </div>
</template>
