<script setup lang="ts">
type QueueProgress = {
  status: string
  progressPercent: number
  message: string
  speed: string | null
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

type QueueItem = string | GifSegment

const props = defineProps<{
  mode: string
  files: string[]
  gifQueue: GifSegment[]
  activityQueueCount: number
  queueProgress: Record<string, QueueProgress>
}>()

const emit = defineEmits<{
  removeFile: [path: string]
  removeGifSegment: [jobId: string]
}>()

const visibleItems = computed<QueueItem[]>(() => props.mode === 'gif' ? props.gifQueue : props.files)

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
          <span v-if="activityQueueCount">
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

    <div
      v-if="visibleItems.length"
      class="max-h-[32rem] space-y-3 overflow-y-auto pr-1"
    >
      <div
        v-for="item in visibleItems"
        :key="typeof item === 'string' ? item : item.jobId"
        class="flex flex-col gap-3 rounded-2xl border border-white/8 bg-white/5 p-4"
      >
        <div class="flex flex-wrap items-center gap-2">
          <p class="truncate text-sm font-medium text-white">
            {{ typeof item === 'string' ? basename(item) : item.label }}
          </p>
          <UBadge
            color="neutral"
            variant="soft"
            :label="detectKind(typeof item === 'string' ? item : item.inputPath)"
          />
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
        <p
          v-if="mode === 'gif' && typeof item !== 'string'"
          class="text-xs text-stone-400"
        >
          {{ item.startSeconds.toFixed(1) }}s -> {{ (item.startSeconds + item.durationSeconds).toFixed(1) }}s • {{ item.fps }} fps • {{ item.width }}px
        </p>
        <div
          v-if="queueItemProgress(item)"
          class="space-y-2"
        >
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
          <p
            v-if="queueItemProgress(item)?.speed"
            class="text-xs text-stone-500"
          >
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
            @click="typeof item === 'string' ? emit('removeFile', item) : emit('removeGifSegment', item.jobId)"
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
</template>
