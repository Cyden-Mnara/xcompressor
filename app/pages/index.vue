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

watch(selectedGifVideo, async () => {
  await updateSelectedGifVideoSrc()
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
          <IntroPanel
            :bootstrap="bootstrap"
            :active-preset="activePreset"
          />
        </aside>

        <main class="grid min-h-0 gap-4">
          <JobConfigurator
            v-model:output-dir="outputDir"
            v-model:mode="mode"
            v-model:preset-id="presetId"
            v-model:video-format="videoFormat"
            v-model:image-format="imageFormat"
            v-model:audio-format="audioFormat"
            v-model:max-parallel-jobs="maxParallelJobs"
            v-model:resize-long-edge="resizeLongEdge"
            :bootstrap="bootstrap"
            :files-count="files.length"
            :mode-options="modeOptions"
            :video-targets="targetFormats('video')"
            :image-targets="targetFormats('image')"
            :audio-targets="targetFormats('audio')"
            :gif-queue-count="gifQueue.length"
            :activity-queue-count="activityQueue.length"
            :processing="processing"
            :cancel-pending="cancelPending"
            :can-run="canRun"
            @pick-files="pickFiles"
            @pick-output-dir="pickOutputDir"
            @clear-queue="clearQueue"
            @add-current-activity="addCurrentActivity"
            @run-batch="runBatch"
            @cancel-batch="cancelBatch"
          />

          <GifEditor
            v-model:gif-options="gifOptions"
            v-model:gif-end-seconds="gifEndSeconds"
            v-model:selected-gif-video="selectedGifVideo"
            :mode="mode"
            :gif-queue-count="gifQueue.length"
            :selected-gif-video-src="selectedGifVideoSrc"
            :selected-gif-video-duration="selectedGifVideoDuration"
            :video-file-options="videoFileOptions"
            :gif-preview-error="gifPreviewError"
            :gif-preview-video="gifPreviewVideo"
            :gif-clip-range-label="gifClipRangeLabel"
            :video-files-count="videoFiles.length"
            :non-video-files-count="nonVideoFiles.length"
            @loaded-metadata="onGifVideoLoaded"
            @pause-preview="setGifStartFromPreview"
            @preview-error="onGifVideoError"
            @open-external="openSelectedGifVideoInSystemPlayer"
            @use-paused-time="setGifStartFromPreview"
            @jump-to-clip-start="syncPreviewToGifStart"
            @set-gif-start="setGifStart"
            @set-gif-end="setGifEnd"
            @add-gif-segment="addGifSegment"
          />

          <div class="grid min-h-0 gap-4 lg:grid-cols-2">
            <ActivityBatch
              :activity-queue="activityQueue"
              :queue-progress="queueProgress"
              @clear-activity-queue="clearActivityQueue"
              @remove-activity-job="removeActivityJob"
            />
            <SourceQueue
              :mode="mode"
              :files="files"
              :gif-queue="gifQueue"
              :activity-queue-count="activityQueue.length"
              :queue-progress="queueProgress"
              @remove-file="removeFile"
              @remove-gif-segment="removeGifSegment"
            />
          </div>
        </main>

        <aside class="grid min-h-0 gap-4">
          <AppUpdates
            :update-status="updateStatus"
            :bootstrap="bootstrap"
            :update-loading="updateLoading"
            :update-installing="updateInstalling"
            :format-update-date="formatUpdateDate"
            @check-for-updates="checkForUpdates"
            @install-update="installUpdate"
          />
          <ResourcePlanner
            :resource-plan="resourcePlan"
            :resource-plan-loading="resourcePlanLoading"
            :live-system-metrics="liveSystemMetrics"
            :eta-caption="etaCaption"
            :estimated-minutes-label="estimatedMinutesLabel"
            :processing="processing"
            :cancel-pending="cancelPending"
            :run-queue-count="runQueueCount"
            :effective-parallel-jobs="effectiveParallelJobs"
            @cancel-batch="cancelBatch"
            @enable-sequential-mode="enableSequentialMode"
          />
          <BatchMonitor
            :overall-progress="overallProgress"
            :activity-queue-count="activityQueue.length"
            :gif-queue-count="gifQueue.length"
            :files-count="files.length"
            :completed-jobs="completedJobs"
            :mode="mode"
            :cancel-pending="cancelPending"
          />
          <BatchOutput :results="results" />
        </aside>
      </div>
    </div>
  </div>
</template>
