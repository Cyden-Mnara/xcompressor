<script setup lang="ts">
import type { AppUiInjection } from '~/utils/app-ui'

type QueueProgress = {
  status: string
  progressPercent: number
  message: string
  speed: string | null
  outputPath: string | null
}

type GifSegment = {
  jobId: string
  inputPath: string
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
  mode: string
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
}

type QueueItem = string | GifSegment | MixedJob

const props = defineProps<{
  mode: string
  files: string[]
  gifQueue: GifSegment[]
  activityQueue: MixedJob[]
  selectedJobId: string
  queueProgress: Record<string, QueueProgress>
}>()

const emit = defineEmits<{
  removeFile: [path: string]
  removeGifSegment: [jobId: string]
  removeActivityJob: [jobId: string]
  selectActivityJob: [jobId: string]
  openOutput: [path: string]
}>()
const appUi = inject('appUi') as AppUiInjection
const ui = computed(() => appUi.value)

const visibleItems = computed<QueueItem[]>(() => {
  if (props.activityQueue.length) {
    return props.activityQueue
  }

  return props.mode === 'gif' ? props.gifQueue : props.files
})

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

function batchJobId(path: string, operation: string) {
  return `${operation}::${path}`
}

function queueItemKey(item: QueueItem) {
  return typeof item === 'string' ? batchJobId(item, props.mode) : item.jobId
}

function queueItemProgress(item: QueueItem) {
  return props.queueProgress[queueItemKey(item)]
}

function itemInputPath(item: QueueItem) {
  return typeof item === 'string' ? item : item.inputPath
}

function itemLabel(item: QueueItem) {
  if (typeof item === 'string') {
    return basename(item)
  }

  return item.label
}

function isMixedJob(item: QueueItem): item is MixedJob {
  return typeof item !== 'string' && 'mode' in item
}

function describeMixedJob(job: MixedJob) {
  if (job.mode === 'gif' && job.gif) {
    return `GIF • ${job.gif.startSeconds.toFixed(1)}s -> ${(job.gif.startSeconds + job.gif.durationSeconds).toFixed(1)}s • ${job.gif.fps} fps • ${job.gif.width}px`
  }

  const kind = detectKind(job.inputPath)
  const target = job.mode === 'extract-audio'
    ? job.audioFormat
    : kind === 'video'
      ? job.videoFormat
      : kind === 'image'
        ? job.imageFormat
        : job.audioFormat
  const resize = job.resizeLongEdge ? ` • ${job.resizeLongEdge}px ${appUi.value.queue.edge}` : ''
  return `${appUi.value.modes[job.mode] ?? job.mode} -> ${target}${resize}`
}

function removeItem(item: QueueItem) {
  if (typeof item === 'string') {
    emit('removeFile', item)
    return
  }

  if (isMixedJob(item)) {
    emit('removeActivityJob', item.jobId)
    return
  }

  emit('removeGifSegment', item.jobId)
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
</script>

<template>
  <UCard :ui="{ root: 'thin-scrollbar overflow-y-auto border border-white/10 bg-stone-950/85 ring-0 lg:max-h-[calc(100dvh-9rem)]' }">
    <template #header>
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
            {{ ui.queue.title }}
          </p>
        </div>
      </div>
    </template>

    <div
      v-if="visibleItems.length"
      class="thin-scrollbar max-h-[60dvh] space-y-3 overflow-y-auto pr-1 lg:max-h-[calc(100dvh-18rem)]"
    >
      <div
        v-for="item in visibleItems"
        :key="typeof item === 'string' ? item : item.jobId"
        class="flex flex-col gap-3 rounded-lg border p-4 transition"
        :class="isMixedJob(item) && item.jobId === selectedJobId ? 'border-amber-300/60 bg-amber-400/10' : 'border-white/8 bg-white/5'"
        role="button"
        tabindex="0"
        @click="isMixedJob(item) && emit('selectActivityJob', item.jobId)"
        @keydown.enter="isMixedJob(item) && emit('selectActivityJob', item.jobId)"
      >
        <div class="flex flex-wrap items-center gap-2">
          <p class="min-w-0 flex-1 truncate text-sm font-medium text-white">
            {{ itemLabel(item) }}
          </p>
          <UBadge
            color="neutral"
            variant="soft"
            :label="ui.media[detectKind(itemInputPath(item))] ?? detectKind(itemInputPath(item))"
          />
          <UBadge
            v-if="isMixedJob(item)"
            color="primary"
            variant="soft"
            :label="ui.modes[item.mode] ?? item.mode"
          />
          <UBadge
            v-if="mode === 'gif' && typeof item === 'string' && detectKind(item) !== 'video'"
            color="warning"
            variant="soft"
            :label="ui.queue.skippedGif"
          />
          <UBadge
            v-if="queueItemProgress(item)"
            :color="statusColor(queueItemProgress(item)?.status)"
            variant="soft"
            :label="ui.status[queueItemProgress(item)?.status || 'queued'] ?? queueItemProgress(item)?.status ?? ui.status.queued"
          />
        </div>
        <p class="truncate text-xs text-stone-500">
          {{ itemInputPath(item) }}
        </p>
        <p
          v-if="isMixedJob(item)"
          class="text-xs text-stone-400"
        >
          {{ describeMixedJob(item) }}
        </p>
        <p
          v-if="mode === 'gif' && typeof item !== 'string' && !isMixedJob(item)"
          class="text-xs text-stone-400"
        >
          {{ item.startSeconds.toFixed(1) }}s -> {{ (item.startSeconds + item.durationSeconds).toFixed(1) }}s • {{ item.fps }} fps • {{ item.width }}px
        </p>
        <div
          v-if="queueItemProgress(item)"
          class="space-y-2"
        >
          <div class="h-1 overflow-hidden rounded-full bg-white/8">
            <div
              class="h-full rounded-full bg-linear-to-r from-amber-400 via-orange-400 to-sky-400 transition-[width] duration-300"
              :style="{ width: `${queueItemProgress(item)?.progressPercent ?? 0}%` }"
            />
          </div>
          <div class="flex flex-wrap items-center justify-between gap-2 text-xs text-stone-400">
            <p>{{ queueItemProgress(item)?.message }}</p>
            <p>{{ queueItemProgress(item)?.progressPercent ?? 0 }}%</p>
          </div>
          <p
            v-if="queueItemProgress(item)?.speed"
            class="text-xs text-stone-500"
          >
            {{ ui.queue.speed }}: {{ queueItemProgress(item)?.speed }}
          </p>
        </div>
        <p
          v-else-if="mode === 'gif' && typeof item === 'string' && detectKind(item) !== 'video'"
          class="text-xs text-amber-300"
        >
          {{ ui.queue.skippedGifBody }}
        </p>
        <div class="flex justify-end gap-2">
          <UButton
            v-if="queueItemProgress(item)?.status === 'completed' && queueItemProgress(item)?.outputPath"
            icon="i-lucide-monitor-play"
            color="neutral"
            variant="soft"
            @click.stop="emit('openOutput', queueItemProgress(item)!.outputPath!)"
          >
            {{ ui.queue.open }}
          </UButton>
          <UButton
            icon="i-lucide-x"
            color="neutral"
            variant="ghost"
            @click.stop="removeItem(item)"
          >
            {{ mode === 'gif' && typeof item !== 'string' && !isMixedJob(item) ? ui.queue.removeClip : ui.queue.remove }}
          </UButton>
        </div>
      </div>
    </div>

    <UEmptyState
      v-else
      icon="i-lucide-clapperboard"
      :title="ui.queue.emptyTitle"
      :description="ui.queue.emptyDescription"
    />
  </UCard>
</template>
