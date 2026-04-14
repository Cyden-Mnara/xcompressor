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

type GuideLanguage = 'en-US' | 'sw-TZ'

type TourStep = {
  title: string
  body: string
  target: string
}

type GuideCopy = {
  workspaceBadge: string
  introBody: string
  setupSuffix: string
  activePresetTitle: string
  defaultPresetLabel: string
  qualityLabel: string
  sizeDeltaLabel: string
  guideTitle: string
  guideSteps: string[]
  detectedNote: string
  workflowTitle: string
  tourTitle: string
  tourSteps: TourStep[]
  tourBack: string
  tourNext: string
  tourDone: string
  tourSkip: string
  themeLabel: string
  languageLabel: string
}

const guideDictionary: Record<GuideLanguage, GuideCopy> = {
  'en-US': {
    workspaceBadge: 'Workspace overview',
    introBody: 'Add media, tune the selected output, then run the queue.',
    setupSuffix: 'setup',
    activePresetTitle: 'Active preset',
    defaultPresetLabel: 'Balanced',
    qualityLabel: 'Quality',
    sizeDeltaLabel: 'Size delta',
    guideTitle: 'Quick guide',
    guideSteps: [
      'Drop files anywhere or use Add media.',
      'Select a queue item to edit that file.',
      'Choose its mode, preset, target format, and output folder.',
      'Run the queue when every selected job is ready.'
    ],
    detectedNote: 'File type and preset follow the selected queue item.',
    workflowTitle: 'GIF workflow',
    tourTitle: 'First run guide',
    tourSteps: [
      {
        title: 'Add media',
        body: 'Drop video, image, or audio files anywhere in the window, or use Add media from this setup panel.',
        target: 'job-config'
      },
      {
        title: 'Read the current context',
        body: 'This guide panel follows the selected file type and preset, so it changes as you move through the queue.',
        target: 'intro-panel'
      },
      {
        title: 'Select a queued item',
        body: 'The queue is where each file lives. Pick an item here to edit its own mode, preset, format, and output folder.',
        target: 'source-queue'
      },
      {
        title: 'Run the queue',
        body: 'When the selected jobs are ready, run the queue from the setup panel. Progress appears beside each item.',
        target: 'job-config'
      }
    ],
    tourBack: 'Back',
    tourNext: 'Next',
    tourDone: 'Done',
    tourSkip: 'Skip',
    themeLabel: 'Theme',
    languageLabel: 'Guide language'
  },
  'sw-TZ': {
    workspaceBadge: 'Muhtasari wa kazi',
    introBody: 'Ongeza faili, rekebisha matokeo ya kipengee kilichochaguliwa, kisha endesha foleni.',
    setupSuffix: 'mipangilio',
    activePresetTitle: 'Preset inayotumika',
    defaultPresetLabel: 'Wastani',
    qualityLabel: 'Ubora',
    sizeDeltaLabel: 'Mabadiliko ya ukubwa',
    guideTitle: 'Mwongozo mfupi',
    guideSteps: [
      'Dondosha faili popote au tumia Add media.',
      'Chagua kipengee kwenye foleni ili kuhariri faili hiyo.',
      'Chagua mode, preset, fomati ya matokeo, na folda ya kuhifadhi.',
      'Endesha foleni kila kazi ikiwa tayari.'
    ],
    detectedNote: 'Aina ya faili na preset hufuata kipengee kilichochaguliwa kwenye foleni.',
    workflowTitle: 'Mtiririko wa GIF',
    tourTitle: 'Mwongozo wa mara ya kwanza',
    tourSteps: [
      {
        title: 'Ongeza faili',
        body: 'Dondosha video, picha, au sauti popote kwenye dirisha, au tumia Add media kwenye paneli hii ya mipangilio.',
        target: 'job-config'
      },
      {
        title: 'Soma muktadha wa sasa',
        body: 'Paneli hii ya mwongozo hufuata aina ya faili na preset ya kipengee kilichochaguliwa.',
        target: 'intro-panel'
      },
      {
        title: 'Chagua kipengee kwenye foleni',
        body: 'Foleni ndiyo sehemu ya kila faili. Chagua kipengee hapa ili kuhariri mode, preset, fomati, na folda yake.',
        target: 'source-queue'
      },
      {
        title: 'Endesha foleni',
        body: 'Kazi zikiwa tayari, endesha foleni kutoka paneli ya mipangilio. Maendeleo yataonekana pembeni ya kila kipengee.',
        target: 'job-config'
      }
    ],
    tourBack: 'Nyuma',
    tourNext: 'Endelea',
    tourDone: 'Maliza',
    tourSkip: 'Ruka',
    themeLabel: 'Mandhari',
    languageLabel: 'Lugha ya mwongozo'
  }
}

const themeOptions = [
  { label: 'System', value: 'system' },
  { label: 'Dark', value: 'dark' },
  { label: 'Light', value: 'light' }
]
const languageOptions = [
  { label: 'English (US)', value: 'en-US' },
  { label: 'Kiswahili (TZ)', value: 'sw-TZ' }
]
const languageStorageKey = 'xcompressor.guideLanguage'
const onboardingStorageKey = 'xcompressor.onboardingSeen'

const files = ref<string[]>([])
const outputDir = ref('')
const mode = ref('compress')
const selectedMediaType = ref('video')
const guideLanguage = ref<GuideLanguage>('en-US')
const showGuideTour = ref(false)
const guideTourStep = ref(0)
const guideTargetRect = ref<DOMRect | null>(null)
const activeWorkspace = ref<'work' | 'development'>('work')
const canOpenDevelopment = import.meta.dev
const colorMode = useColorMode()
const presetId = ref('balanced')
const resizeLongEdge = ref(1280)
const maxParallelJobs = ref(1)
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
const activeRunJobIds = ref<string[]>([])
const gifSegments = ref<GifSegment[]>([])
const activityQueue = ref<MixedJob[]>([])
const selectedQueueJobId = ref('')
const selectedGifVideo = ref('')
const selectedGifVideoSrc = ref('')
const selectedGifVideoDuration = ref(0)
const gifPreviewError = ref('')
const gifPreviewVideo = ref<HTMLVideoElement | null>(null)
const gifPlayCaptureActive = ref(false)
const dragActive = ref(false)
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
const selectedQueueJob = computed(() => activityQueue.value.find(job => job.jobId === selectedQueueJobId.value) ?? null)
const activeIntroPreset = computed(() =>
  bootstrap.value?.presets.find(preset => preset.id === (selectedQueueJob.value?.presetId ?? presetId.value))
)
const guideCopy = computed(() => guideDictionary[guideLanguage.value])
const activeTourStep = computed(() => guideCopy.value.tourSteps[guideTourStep.value] ?? {
  title: '',
  body: '',
  target: ''
})
const isLastTourStep = computed(() => guideTourStep.value >= guideCopy.value.tourSteps.length - 1)
const guideSpotlightStyle = computed(() => {
  const rect = guideTargetRect.value
  if (!rect) {
    return {
      opacity: '0'
    }
  }

  const padding = 8
  return {
    left: `${Math.max(rect.left - padding, 8)}px`,
    top: `${Math.max(rect.top - padding, 8)}px`,
    width: `${Math.min(rect.width + padding * 2, window.innerWidth - Math.max(rect.left - padding, 8) - 8)}px`,
    height: `${Math.min(rect.height + padding * 2, window.innerHeight - Math.max(rect.top - padding, 8) - 8)}px`,
    opacity: '1'
  }
})
const guideCardStyle = computed(() => {
  const rect = guideTargetRect.value
  const cardWidth = 448

  if (!rect) {
    return {
      left: '50%',
      top: '50%',
      transform: 'translate(-50%, -50%)'
    }
  }

  const left = Math.min(Math.max(rect.left, 16), Math.max(window.innerWidth - cardWidth - 16, 16))
  const spaceBelow = window.innerHeight - rect.bottom
  const top = spaceBelow > 240
    ? rect.bottom + 18
    : Math.max(rect.top - 260, 16)

  return {
    left: `${left}px`,
    top: `${top}px`,
    transform: 'none'
  }
})
const videoFiles = computed(() => files.value.filter(path => detectKind(path) === 'video'))
const videoFileOptions = computed(() =>
  videoFiles.value.map(path => ({
    label: basename(path),
    value: path
  }))
)
const nonVideoFiles = computed(() => files.value.filter(path => detectKind(path) !== 'video'))
const gifQueue = computed(() => gifSegments.value)
const selectedQueueMediaKind = computed(() => selectedQueueJob.value ? detectKind(selectedQueueJob.value.inputPath) : selectedMediaType.value)
const runQueue = computed<QueueItem[] | MixedJob[]>(() => {
  if (activityQueue.value.length) {
    return activityQueue.value
  }

  return mode.value === 'gif' ? gifQueue.value : files.value
})
const runQueueCount = computed(() => processing.value && activeRunJobIds.value.length ? activeRunJobIds.value.length : runQueue.value.length)
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
  if (processing.value) {
    return false
  }

  if (resourcePlan.value?.shouldUseSequential && effectiveParallelJobs.value > 1) {
    return false
  }

  if (activityQueue.value.length) {
    return activityQueue.value.every(job => job.outputDir.length > 0)
  }

  if (!outputDir.value.length) {
    return false
  }

  if (mode.value === 'gif') {
    return gifQueue.value.length > 0
  }

  return files.value.length > 0
})
const overallProgress = computed(() => {
  const progressKeys = activeProgressKeys.value
  if (!progressKeys.length) {
    return 0
  }

  const total = progressKeys.reduce((sum, key) => sum + (queueProgress.value[key]?.progressPercent ?? 0), 0)
  return Math.round(total / progressKeys.length)
})
const activeProgressKeys = computed(() => {
  if (processing.value && activeRunJobIds.value.length) {
    return activeRunJobIds.value
  }

  if (activityQueue.value.length) {
    return activityQueue.value.map(job => job.jobId)
  }

  return mode.value === 'gif'
    ? gifQueue.value.map(segment => segment.jobId)
    : files.value.map(path => batchJobId(path, mode.value))
})
const completedJobs = computed(() =>
  activeProgressKeys.value.filter(key => isTerminalStatus(queueProgress.value[key]?.status)).length
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
const ramUsedPercent = computed(() => {
  const metrics = liveSystemMetrics.value
  if (!metrics?.totalMemoryMb) {
    return null
  }

  return Math.round((metrics.usedMemoryMb / metrics.totalMemoryMb) * 100)
})
const cpuUsedPercent = computed(() => {
  const metrics = liveSystemMetrics.value
  if (!metrics) {
    return null
  }

  return Math.round(metrics.cpuUsagePercent)
})

useHead({
  htmlAttrs: {
    lang: computed(() => guideLanguage.value)
  }
})

let unlistenBatchProgress: null | (() => void) = null
let unlistenTauriDragDrop: null | (() => void) = null
let liveMetricsInterval: ReturnType<typeof setInterval> | null = null
let liveMetricsStartTimer: ReturnType<typeof setTimeout> | null = null
let resourcePlanRequestId = 0

function normalizeGuideLanguage(value: string | undefined): GuideLanguage | null {
  if (!value) {
    return null
  }

  if (value in guideDictionary) {
    return value as GuideLanguage
  }

  const lowerValue = value.toLowerCase()
  if (lowerValue.startsWith('sw')) {
    return 'sw-TZ'
  }

  if (lowerValue.startsWith('en')) {
    return 'en-US'
  }

  return null
}

function detectGuideLanguage() {
  if (!import.meta.client) {
    return 'en-US'
  }

  const storedLanguage = normalizeGuideLanguage(localStorage.getItem(languageStorageKey) ?? undefined)
  if (storedLanguage) {
    return storedLanguage
  }

  const browserLanguages = navigator.languages?.length ? navigator.languages : [navigator.language]
  return browserLanguages
    .map(language => normalizeGuideLanguage(language))
    .find((language): language is GuideLanguage => Boolean(language)) ?? 'en-US'
}

function persistGuideLanguage() {
  if (import.meta.client) {
    localStorage.setItem(languageStorageKey, guideLanguage.value)
  }
}

function finishGuideTour() {
  showGuideTour.value = false
  guideTourStep.value = 0
  guideTargetRect.value = null

  if (import.meta.client) {
    localStorage.setItem(onboardingStorageKey, 'true')
  }
}

function nextGuideTourStep() {
  if (isLastTourStep.value) {
    finishGuideTour()
    return
  }

  guideTourStep.value += 1
}

function previousGuideTourStep() {
  guideTourStep.value = Math.max(guideTourStep.value - 1, 0)
}

async function updateGuideTourTarget() {
  if (!import.meta.client || !showGuideTour.value || !activeTourStep.value.target) {
    guideTargetRect.value = null
    return
  }

  await nextTick()

  const target = document.querySelector<HTMLElement>(`[data-guide-target="${activeTourStep.value.target}"]`)
  if (!target) {
    guideTargetRect.value = null
    return
  }

  target.scrollIntoView({ block: 'nearest', inline: 'nearest' })
  await nextTick()
  guideTargetRect.value = target.getBoundingClientRect()
}

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

function mediaExtensions(kind: string) {
  if (kind === 'video') {
    return ['mp4', 'mov', 'mkv', 'avi', 'webm']
  }

  if (kind === 'image') {
    return ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'tiff']
  }

  if (kind === 'audio') {
    return ['mp3', 'wav', 'aac', 'm4a', 'flac', 'opus', 'ogg']
  }

  return ['mp4', 'mov', 'mkv', 'avi', 'webm', 'png', 'jpg', 'jpeg', 'webp', 'bmp', 'tiff', 'mp3', 'wav', 'aac', 'm4a', 'flac', 'opus', 'ogg']
}

function batchJobId(path: string, operation: string) {
  return `${operation}::${path}`
}

function queueJobId(path: string) {
  return `mixed::${path}`
}

function syncActiveMediaTypeFromPath(path: string | undefined) {
  if (!path) {
    return
  }

  const mediaKind = detectKind(path)
  if (mediaKind !== 'unknown') {
    selectedMediaType.value = mediaKind
  }
}

function updateSelectedQueueJob(patch: Partial<MixedJob>) {
  const selectedJob = selectedQueueJob.value
  if (!selectedJob) {
    return
  }

  activityQueue.value = activityQueue.value.map((job) => {
    if (job.jobId !== selectedJob.jobId) {
      return job
    }

    const nextJob = {
      ...job,
      ...patch
    }
    const mediaKind = detectKind(nextJob.inputPath)

    if (nextJob.mode !== 'gif' || mediaKind !== 'video') {
      nextJob.gif = null
      nextJob.outputSuffix = null
    } else if (!nextJob.gif) {
      nextJob.gif = {
        startSeconds: gifOptions.startSeconds,
        durationSeconds: gifOptions.durationSeconds,
        fps: gifOptions.fps,
        width: gifOptions.width
      }
      nextJob.outputSuffix = `gif-${Date.now().toString(36)}`
    }

    nextJob.resizeLongEdge = nextJob.mode === 'gif' || mediaKind === 'audio'
      ? null
      : nextJob.resizeLongEdge
    nextJob.label = `${basename(nextJob.inputPath)} • ${nextJob.mode}`

    return nextJob
  })
}

function defaultModeForPath(path: string) {
  const mediaKind = detectKind(path)
  if (mode.value === 'gif') {
    return mediaKind === 'video' ? 'gif' : 'compress'
  }

  return mode.value
}

function defaultGifForPath(path: string) {
  if (defaultModeForPath(path) !== 'gif' || detectKind(path) !== 'video') {
    return null
  }

  return {
    startSeconds: gifOptions.startSeconds,
    durationSeconds: gifOptions.durationSeconds,
    fps: gifOptions.fps,
    width: gifOptions.width
  }
}

function makeDefaultQueueJob(path: string): MixedJob {
  const nextMode = defaultModeForPath(path)
  const mediaKind = detectKind(path)

  return {
    inputPath: path,
    outputDir: outputDir.value,
    mode: nextMode,
    presetId: presetId.value,
    videoFormat: videoFormat.value,
    imageFormat: imageFormat.value,
    audioFormat: audioFormat.value,
    jobId: queueJobId(path),
    label: `${basename(path)} • ${nextMode}`,
    resizeLongEdge: nextMode === 'gif' || mediaKind === 'audio' ? null : resizeLongEdge.value,
    gif: defaultGifForPath(path),
    outputSuffix: nextMode === 'gif' ? `gif-${Date.now().toString(36)}` : null,
    overwrite: true
  }
}

async function addPathsToQueue(paths: string[]) {
  const supportedPaths = paths.filter(path => detectKind(path) !== 'unknown')
  const uniqueSupportedPaths = Array.from(new Set(supportedPaths))
  const existingPaths = new Set(files.value)
  const newPaths = uniqueSupportedPaths.filter(path => !existingPaths.has(path))
  const skippedCount = paths.length - newPaths.length

  if (newPaths.length) {
    files.value = Array.from(new Set([...files.value, ...newPaths]))
    const existingJobIds = new Set(activityQueue.value.map(job => job.jobId))
    const nextJobs = newPaths
      .map(path => makeDefaultQueueJob(path))
      .filter(job => !existingJobIds.has(job.jobId))

    activityQueue.value = [...activityQueue.value, ...nextJobs]
    selectedQueueJobId.value = nextJobs.at(-1)?.jobId ?? selectedQueueJobId.value
    syncActiveMediaTypeFromPath(nextJobs.at(-1)?.inputPath ?? newPaths.at(-1))
    syncSelectedGifVideo()
    await updateSelectedGifVideoSrc()
  }

  if (newPaths.length || skippedCount) {
    toast.add({
      title: newPaths.length ? 'Files added' : 'No files added',
      description: `${newPaths.length} added${skippedCount ? `, ${skippedCount} skipped` : ''}.`,
      icon: newPaths.length ? 'i-lucide-check-circle-2' : 'i-lucide-circle-alert',
      color: newPaths.length ? 'success' : 'warning',
      duration: 5000
    })
  }
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

function currentGifPreviewTime() {
  return Number((gifPreviewVideo.value?.currentTime ?? gifOptions.startSeconds).toFixed(2))
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
        name: 'Supported media',
        extensions: mediaExtensions('all')
      }
    ]
  })

  if (!selection) {
    return
  }

  const nextPaths = Array.isArray(selection) ? selection : [selection]
  await addPathsToQueue(nextPaths)
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
  activityQueue.value = activityQueue.value.filter(job => job.inputPath !== path)
  if (selectedQueueJob.value?.inputPath === path) {
    selectedQueueJobId.value = activityQueue.value[0]?.jobId ?? ''
  }
  syncActiveMediaTypeFromPath(selectedQueueJob.value?.inputPath ?? files.value[0])
  syncSelectedGifVideo()
}

function clearQueue() {
  files.value = []
  results.value = []
  queueProgress.value = {}
  gifSegments.value = []
  activityQueue.value = []
  selectedQueueJobId.value = ''
  selectedGifVideo.value = ''
  selectedGifVideoSrc.value = ''
  selectedGifVideoDuration.value = 0
  gifPlayCaptureActive.value = false
}

function clearCurrentRunState() {
  results.value = []
  queueProgress.value = {}
  activeRunJobIds.value = []
  cancelPending.value = false
}

function isCompletedStatus(status: string | undefined) {
  return status === 'completed'
}

function pruneCompletedQueueItems() {
  if (!Object.keys(queueProgress.value).length) {
    return 0
  }

  if (activityQueue.value.length) {
    const originalCount = activityQueue.value.length
    activityQueue.value = activityQueue.value.filter(job => !isCompletedStatus(queueProgress.value[job.jobId]?.status))
    return originalCount - activityQueue.value.length
  } else if (mode.value === 'gif') {
    const originalCount = gifSegments.value.length
    gifSegments.value = gifSegments.value.filter(segment => !isCompletedStatus(queueProgress.value[segment.jobId]?.status))
    return originalCount - gifSegments.value.length
  }

  const originalCount = files.value.length
  files.value = files.value.filter(path => !isCompletedStatus(queueProgress.value[batchJobId(path, mode.value)]?.status))
  return originalCount - files.value.length
}

function pendingQueueCount(hadActivityQueue: boolean) {
  if (hadActivityQueue) {
    return activityQueue.value.length
  }

  return mode.value === 'gif' ? gifQueue.value.length : files.value.length
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

function markActiveJobsFailed(message: string) {
  const failedProgress = activeRunJobIds.value.reduce<Record<string, QueueProgress>>((nextProgress, jobId) => {
    const previousProgress = queueProgress.value[jobId]
    nextProgress[jobId] = {
      jobId,
      label: previousProgress?.label ?? null,
      mediaKind: previousProgress?.mediaKind ?? 'system',
      operation: previousProgress?.operation ?? mode.value,
      status: 'failed',
      progressPercent: previousProgress?.progressPercent ?? 0,
      outputPath: previousProgress?.outputPath ?? null,
      message,
      speed: previousProgress?.speed ?? null
    }
    return nextProgress
  }, {})

  queueProgress.value = {
    ...queueProgress.value,
    ...failedProgress
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
  if (processing.value) {
    return
  }

  if (!files.value.length && !gifSegments.value.length && !activityQueue.value.length) {
    resourcePlan.value = null
    resourcePlanLoading.value = false
    return
  }

  const requestId = ++resourcePlanRequestId
  resourcePlanLoading.value = true

  try {
    const plan = await tauriInvoke<ResourcePlan>('analyze_resource_plan', {
      request: buildResourcePayload()
    })
    if (requestId === resourcePlanRequestId) {
      resourcePlan.value = plan
    }
  } catch {
    if (requestId === resourcePlanRequestId) {
      resourcePlan.value = null
    }
  } finally {
    if (requestId === resourcePlanRequestId) {
      resourcePlanLoading.value = false
    }
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
  }, 4000)
}

function stopLiveMetricsPolling() {
  if (liveMetricsInterval) {
    clearInterval(liveMetricsInterval)
    liveMetricsInterval = null
  }

  if (liveMetricsStartTimer) {
    clearTimeout(liveMetricsStartTimer)
    liveMetricsStartTimer = null
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

  if (activityQueue.value.length) {
    activityQueue.value = activityQueue.value.map((job) => {
      const mediaKind = detectKind(job.inputPath)
      const nextMode = mediaKind === 'video' ? mode.value : mode.value === 'gif' ? 'compress' : mode.value
      return {
        ...job,
        outputDir: outputDir.value,
        mode: nextMode,
        presetId: presetId.value,
        videoFormat: videoFormat.value,
        imageFormat: imageFormat.value,
        audioFormat: audioFormat.value,
        resizeLongEdge: nextMode === 'gif' || mediaKind === 'audio' ? null : resizeLongEdge.value,
        gif: nextMode === 'gif' && mediaKind === 'video'
          ? {
              startSeconds: gifOptions.startSeconds,
              durationSeconds: gifOptions.durationSeconds,
              fps: gifOptions.fps,
              width: gifOptions.width
            }
          : null,
        outputSuffix: nextMode === 'gif' && mediaKind === 'video' ? `gif-${Date.now().toString(36)}` : null,
        label: `${basename(job.inputPath)} • ${nextMode}`
      }
    })
    toast.add({
      title: 'Defaults applied',
      description: `${activityQueue.value.length} queued jobs now use the current settings.`,
      icon: 'i-lucide-check-circle-2',
      color: 'success',
      duration: 5000
    })
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
  const removedJob = activityQueue.value.find(job => job.jobId === jobId)
  activityQueue.value = activityQueue.value.filter(job => job.jobId !== jobId)
  if (removedJob && !activityQueue.value.some(job => job.inputPath === removedJob.inputPath)) {
    files.value = files.value.filter(path => path !== removedJob.inputPath)
    gifSegments.value = gifSegments.value.filter(segment => segment.inputPath !== removedJob.inputPath)
  }
  if (selectedQueueJobId.value === jobId) {
    selectedQueueJobId.value = activityQueue.value[0]?.jobId ?? ''
  }
  syncActiveMediaTypeFromPath(selectedQueueJob.value?.inputPath ?? files.value[0])
  syncSelectedGifVideo()
}

function clearActivityQueue() {
  activityQueue.value = []
  selectedQueueJobId.value = ''
}

async function handleDrop(event: DragEvent) {
  event.preventDefault()
  dragActive.value = false
  const droppedFiles = Array.from(event.dataTransfer?.files ?? [])
  const paths = droppedFiles
    .map(file => (file as File & { path?: string }).path ?? '')
    .filter(Boolean)

  await addPathsToQueue(paths)
}

function handleDragOver(event: DragEvent) {
  event.preventDefault()
  dragActive.value = true
}

function handleDragLeave(event: DragEvent) {
  if (event.currentTarget === event.target) {
    dragActive.value = false
  }
}

async function registerTauriDragDrop() {
  if (unlistenTauriDragDrop) {
    return
  }

  const { getCurrentWebview } = await import('@tauri-apps/api/webview')
  unlistenTauriDragDrop = await getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === 'enter' || event.payload.type === 'over') {
      dragActive.value = true
      return
    }

    if (event.payload.type === 'drop') {
      dragActive.value = false
      void addPathsToQueue(event.payload.paths)
      return
    }

    dragActive.value = false
  })
}

async function registerBatchListener() {
  if (unlistenBatchProgress) {
    return
  }

  const { listen } = await import('@tauri-apps/api/event')
  unlistenBatchProgress = await listen<BatchProgressEvent>('batch-progress', ({ payload }) => {
    updateQueueProgress(payload)
  })
}

async function runBatch() {
  await registerBatchListener()
  const hadActivityQueue = activityQueue.value.length > 0
  const prunedCount = pruneCompletedQueueItems()

  if (prunedCount) {
    toast.add({
      title: 'Completed jobs removed',
      description: `${prunedCount} completed ${prunedCount === 1 ? 'job was' : 'jobs were'} removed before rerun.`,
      icon: 'i-lucide-list-checks',
      color: 'success',
      duration: 5000
    })
  }

  if (!pendingQueueCount(hadActivityQueue)) {
    clearCurrentRunState()
    toast.add({
      title: 'No pending jobs',
      description: 'Everything in the queue has already completed.',
      icon: 'i-lucide-circle-check',
      color: 'neutral',
      duration: 5000
    })
    return
  }

  clearCurrentRunState()
  currentRunId.value = `run-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
  activeRunJobIds.value = activityQueue.value.length
    ? activityQueue.value.map(job => job.jobId)
    : mode.value === 'gif'
      ? gifQueue.value.map(segment => segment.jobId)
      : files.value.map(path => batchJobId(path, mode.value))
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
    const message = String(error)
    markActiveJobsFailed(message)
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
      message
    }]
  } finally {
    processing.value = false
    cancelPending.value = false
    currentRunId.value = ''
    activeRunJobIds.value = []
    void refreshResourcePlan()
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

onMounted(() => {
  guideLanguage.value = detectGuideLanguage()
  showGuideTour.value = import.meta.client && localStorage.getItem(onboardingStorageKey) !== 'true'
  if (import.meta.client) {
    window.addEventListener('resize', updateGuideTourTarget)
    window.addEventListener('scroll', updateGuideTourTarget, true)
    void updateGuideTourTarget()
  }
  void registerTauriDragDrop()
  void loadBootstrap().then(() => {
    void refreshResourcePlan()
  })
  syncSelectedGifVideo()
  if (selectedGifVideo.value) {
    void updateSelectedGifVideoSrc()
  }
  liveMetricsStartTimer = setTimeout(() => {
    void refreshLiveSystemMetrics()
    startLiveMetricsPolling()
  }, 1500)
})

onBeforeUnmount(() => {
  unlistenBatchProgress?.()
  unlistenTauriDragDrop?.()
  if (import.meta.client) {
    window.removeEventListener('resize', updateGuideTourTarget)
    window.removeEventListener('scroll', updateGuideTourTarget, true)
  }
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

watch(guideLanguage, () => {
  persistGuideLanguage()
  guideTourStep.value = Math.min(guideTourStep.value, guideCopy.value.tourSteps.length - 1)
  void updateGuideTourTarget()
})

watch([showGuideTour, guideTourStep, activeWorkspace], () => {
  void updateGuideTourTarget()
})

watch(outputDir, (nextOutputDir) => {
  if (!nextOutputDir) {
    return
  }

  activityQueue.value = activityQueue.value.map(job => ({
    ...job,
    outputDir: job.outputDir || nextOutputDir
  }))
})

watch([files, gifSegments, activityQueue, maxParallelJobs, outputDir, presetId, resizeLongEdge, videoFormat, imageFormat, audioFormat], async () => {
  await refreshResourcePlan()
}, { deep: true })

function onGifVideoLoaded(event: Event) {
  const target = event.target as HTMLVideoElement
  selectedGifVideoDuration.value = Number.isFinite(target.duration) ? target.duration : 0
  gifPreviewVideo.value = target
  gifPreviewError.value = ''
  gifPlayCaptureActive.value = false
  clampGifRange()
}

function onGifPreviewPlay() {
  setGifStart(currentGifPreviewTime())
  gifPlayCaptureActive.value = true
}

function onGifPreviewPause() {
  if (!gifPlayCaptureActive.value) {
    return
  }

  gifPlayCaptureActive.value = false
  const endSeconds = currentGifPreviewTime()
  if (endSeconds > gifOptions.startSeconds + 0.1) {
    setGifEnd(endSeconds)
  }
}

function onGifVideoError() {
  gifPreviewVideo.value = null
  gifPlayCaptureActive.value = false
  gifPreviewError.value = 'Embedded preview failed in the desktop webview for this file or codec. Open it in the system player instead.'
}
</script>

<template>
  <div
    class="relative min-h-screen text-stone-100"
    @drop="handleDrop"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
  >
    <div
      v-if="dragActive"
      class="pointer-events-none fixed inset-0 z-50 flex items-center justify-center border-3 border-dashed border-amber-300 bg-black/70"
    >
      <div class="rounded-lg border border-amber-300/40 bg-stone-950 p-6 text-center shadow-2xl">
        <p class="text-xl font-semibold text-white">
          Drop files to add them to the queue
        </p>
        <p class="mt-2 text-sm text-stone-300">
          Videos, images, and audio get default settings that you can edit per file.
        </p>
      </div>
    </div>
    <div
      v-if="showGuideTour"
      class="fixed inset-0 z-[60] bg-black/70"
    >
      <div
        class="pointer-events-none fixed rounded-lg border-2 border-amber-300 shadow-[0_0_0_9999px_rgba(0,0,0,0.55),0_0_28px_rgba(251,191,36,0.45)] transition-all duration-200"
        :style="guideSpotlightStyle"
      />
      <div
        class="fixed w-[calc(100vw-2rem)] max-w-md rounded-lg border border-amber-300/35 bg-stone-950 p-5 shadow-2xl transition-all duration-200"
        :style="guideCardStyle"
      >
        <div class="flex items-start justify-between gap-4">
          <div>
            <p class="text-xs font-semibold uppercase tracking-[0.25em] text-amber-300">
              {{ guideCopy.tourTitle }}
            </p>
            <h2 class="mt-3 text-2xl font-semibold text-white">
              {{ activeTourStep.title }}
            </h2>
          </div>
          <UBadge
            color="neutral"
            variant="soft"
            :label="`${guideTourStep + 1}/${guideCopy.tourSteps.length}`"
          />
        </div>
        <p class="mt-4 text-sm leading-6 text-stone-300">
          {{ activeTourStep.body }}
        </p>
        <div class="mt-5 flex flex-wrap items-center justify-between gap-3">
          <label class="flex items-center gap-2 text-xs font-medium text-stone-300">
            <span>{{ guideCopy.languageLabel }}</span>
            <USelect
              v-model="guideLanguage"
              :items="languageOptions"
              size="sm"
              class="w-40"
              :aria-label="guideCopy.languageLabel"
            />
          </label>
          <div class="flex flex-wrap gap-2">
            <UButton
              color="neutral"
              variant="ghost"
              @click="finishGuideTour"
            >
              {{ guideCopy.tourSkip }}
            </UButton>
            <UButton
              color="neutral"
              variant="soft"
              :disabled="guideTourStep === 0"
              @click="previousGuideTourStep"
            >
              {{ guideCopy.tourBack }}
            </UButton>
            <UButton
              color="primary"
              @click="nextGuideTourStep"
            >
              {{ isLastTourStep ? guideCopy.tourDone : guideCopy.tourNext }}
            </UButton>
          </div>
        </div>
      </div>
    </div>
    <div class="m-auto h-full max-w-425 px-4 py-4 lg:px-6">
      <div
        v-if="canOpenDevelopment"
        class="mb-4 flex flex-wrap items-center justify-between gap-3"
      >
        <div class="flex flex-wrap gap-2">
          <UButton
            :color="activeWorkspace === 'work' ? 'primary' : 'neutral'"
            :variant="activeWorkspace === 'work' ? 'solid' : 'soft'"
            icon="i-lucide-layout-dashboard"
            @click="activeWorkspace = 'work'"
          >
            Work
          </UButton>
          <UButton
            :color="activeWorkspace === 'development' ? 'primary' : 'neutral'"
            :variant="activeWorkspace === 'development' ? 'solid' : 'soft'"
            icon="i-lucide-terminal"
            @click="activeWorkspace = 'development'"
          >
            Development
          </UButton>
        </div>
        <p class="text-sm text-stone-400">
          One job runs at a time to keep the machine responsive.
        </p>
        <UButton
          icon="i-lucide-refresh-cw"
          color="neutral"
          variant="soft"
          size="sm"
          :loading="updateLoading"
          @click="checkForUpdates"
        >
          Check updates
        </UButton>
        <label class="flex items-center gap-2 text-xs font-medium text-stone-300">
          <span>{{ guideCopy.themeLabel }}</span>
          <USelect
            v-model="colorMode.preference"
            :items="themeOptions"
            size="sm"
            class="w-32"
            :aria-label="guideCopy.themeLabel"
          />
        </label>
        <label class="flex items-center gap-2 text-xs font-medium text-stone-300">
          <span>{{ guideCopy.languageLabel }}</span>
          <USelect
            v-model="guideLanguage"
            :items="languageOptions"
            size="sm"
            class="w-40"
            :aria-label="guideCopy.languageLabel"
          />
        </label>
      </div>
      <div
        v-else
        class="mb-4 flex justify-end gap-2"
      >
        <UButton
          icon="i-lucide-refresh-cw"
          color="neutral"
          variant="soft"
          size="sm"
          :loading="updateLoading"
          @click="checkForUpdates"
        >
          Check updates
        </UButton>
        <label class="flex items-center gap-2 text-xs font-medium text-stone-300">
          <span>{{ guideCopy.themeLabel }}</span>
          <USelect
            v-model="colorMode.preference"
            :items="themeOptions"
            size="sm"
            class="w-32"
            :aria-label="guideCopy.themeLabel"
          />
        </label>
        <label class="flex items-center gap-2 text-xs font-medium text-stone-300">
          <span>{{ guideCopy.languageLabel }}</span>
          <USelect
            v-model="guideLanguage"
            :items="languageOptions"
            size="sm"
            class="w-40"
            :aria-label="guideCopy.languageLabel"
          />
        </label>
      </div>

      <div
        v-if="activeWorkspace === 'work'"
        class="grid min-h-0 gap-4 lg:max-h-[calc(100dvh-5rem)] xl:grid-cols-[300px_minmax(0,1fr)_440px]"
      >
        <aside
          class="grid min-h-0 gap-4"
          data-guide-target="intro-panel"
        >
          <IntroPanel
            :bootstrap="bootstrap"
            :active-preset="activeIntroPreset"
            :active-media-type="selectedQueueMediaKind"
            :guide-copy="guideCopy"
          />
        </aside>

        <main
          class="grid min-h-0 gap-4 lg:max-h-[calc(100dvh-5rem)]"
          data-guide-target="job-config"
        >
          <JobConfigurator
            v-model:output-dir="outputDir"
            v-model:mode="mode"
            :bootstrap="bootstrap"
            :files-count="files.length"
            :video-targets="targetFormats('video')"
            :image-targets="targetFormats('image')"
            :audio-targets="targetFormats('audio')"
            :gif-queue-count="gifQueue.length"
            :activity-queue-count="activityQueue.length"
            :processing="processing"
            :cancel-pending="cancelPending"
            :can-run="canRun"
            :selected-job="selectedQueueJob"
            :selected-media-kind="selectedQueueMediaKind"
            @pick-files="pickFiles"
            @pick-output-dir="pickOutputDir"
            @clear-queue="clearQueue"
            @add-current-activity="addCurrentActivity"
            @run-batch="runBatch"
            @cancel-batch="cancelBatch"
            @update-selected-job="updateSelectedQueueJob"
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
            @play-preview="onGifPreviewPlay"
            @pause-preview="onGifPreviewPause"
            @preview-error="onGifVideoError"
            @open-external="openSelectedGifVideoInSystemPlayer"
            @jump-to-clip-start="syncPreviewToGifStart"
            @set-gif-start="setGifStart"
            @set-gif-end="setGifEnd"
            @add-gif-segment="addGifSegment"
          />
        </main>

        <aside
          class="grid min-h-0 grid-rows-[auto_minmax(0,1fr)] gap-3"
          data-guide-target="source-queue"
        >
          <div class="rounded-lg border border-white/10 bg-stone-950/85 p-3">
            <div class="grid grid-cols-3 gap-2 text-center">
              <div class="rounded-lg bg-white/5 p-2">
                <p class="text-[0.65rem] font-semibold uppercase tracking-[0.18em] text-stone-500">
                  RAM
                </p>
                <p class="mt-1 text-lg font-semibold text-white">
                  {{ ramUsedPercent === null || ramUsedPercent === undefined ? 'n/a' : `${ramUsedPercent}%` }}
                </p>
              </div>
              <div class="rounded-lg bg-white/5 p-2">
                <p class="text-[0.65rem] font-semibold uppercase tracking-[0.18em] text-stone-500">
                  CPU
                </p>
                <p class="mt-1 text-lg font-semibold text-white">
                  {{ cpuUsedPercent === null || cpuUsedPercent === undefined ? 'n/a' : `${cpuUsedPercent}%` }}
                </p>
              </div>
              <div class="rounded-lg bg-white/5 p-2">
                <p class="text-[0.65rem] font-semibold uppercase tracking-[0.18em] text-stone-500">
                  ETA
                </p>
                <p class="mt-1 text-lg font-semibold text-white">
                  {{ estimatedMinutesLabel }}
                </p>
              </div>
            </div>
          </div>
          <SourceQueue
            :mode="mode"
            :files="files"
            :gif-queue="gifQueue"
            :activity-queue="activityQueue"
            :selected-job-id="selectedQueueJobId"
            :queue-progress="queueProgress"
            @remove-file="removeFile"
            @remove-gif-segment="removeGifSegment"
            @remove-activity-job="removeActivityJob"
            @select-activity-job="selectedQueueJobId = $event"
          />
        </aside>
      </div>

      <div
        v-else
        class="grid min-h-0 gap-4 lg:max-h-[calc(100dvh-5rem)] xl:grid-cols-[380px_minmax(0,1fr)]"
      >
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
        </aside>
        <main class="min-h-0">
          <div class="grid min-h-0 gap-4 xl:grid-cols-2">
            <ActivityBatch
              :activity-queue="activityQueue"
              :queue-progress="queueProgress"
              @clear-activity-queue="clearActivityQueue"
              @remove-activity-job="removeActivityJob"
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
          </div>
          <div class="mt-4">
            <BatchOutput :results="results" />
          </div>
        </main>
      </div>
    </div>
  </div>
</template>
