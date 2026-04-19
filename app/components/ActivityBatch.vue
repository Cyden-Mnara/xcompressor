<script setup lang="ts">
import type { AppUiInjection } from '~/utils/app-ui'

type QueueProgress = {
  status: string
  progressPercent: number
  message: string
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

const props = defineProps<{
  activityQueue: MixedJob[]
  queueProgress: Record<string, QueueProgress>
}>()

const emit = defineEmits<{
  clearActivityQueue: []
  removeActivityJob: [jobId: string]
}>()
const appUi = inject('appUi') as AppUiInjection
const ui = computed(() => appUi.value)

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

function mixedJobProgress(item: MixedJob) {
  return props.queueProgress[item.jobId]
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

function describeActivity(job: MixedJob) {
  if (job.mode === 'gif' && job.gif) {
    const end = job.gif.startSeconds + job.gif.durationSeconds
    return `${job.gif.startSeconds.toFixed(1)}s -> ${end.toFixed(1)}s • ${job.gif.fps} fps • ${job.gif.width}px`
  }

  const target = job.mode === 'extract-audio'
    ? job.audioFormat
    : detectKind(job.inputPath) === 'video'
      ? job.videoFormat
      : detectKind(job.inputPath) === 'image'
        ? job.imageFormat
        : job.audioFormat
  const resize = job.resizeLongEdge ? ` • ${job.resizeLongEdge}px ${appUi.value.activity.edge}` : ''
  return `${appUi.value.modes[job.mode] ?? job.mode} -> ${target}${resize}`
}
</script>

<template>
  <UCard :ui="{ root: 'thin-scrollbar overflow-y-auto border border-white/10 bg-stone-950/85 ring-0 lg:max-h-[calc(100dvh-5rem)]' }">
    <template #header>
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
            {{ ui.activity.title }}
          </p>
          <h2 class="mt-2 text-xl font-semibold text-white">
            {{ ui.activity.subtitle }}
          </h2>
        </div>
        <div class="flex items-center gap-3">
          <UBadge
            color="primary"
            variant="soft"
            :label="`${activityQueue.length} ${ui.activity.queued}`"
          />
          <UButton
            icon="i-lucide-trash-2"
            color="error"
            variant="ghost"
            :disabled="!activityQueue.length"
            @click="emit('clearActivityQueue')"
          >
            {{ ui.activity.clear }}
          </UButton>
        </div>
      </div>
    </template>

    <div
      v-if="activityQueue.length"
      class="thin-scrollbar max-h-[60dvh] space-y-3 overflow-y-auto pr-1 lg:max-h-[calc(100dvh-17rem)]"
    >
      <div
        v-for="job in activityQueue"
        :key="job.jobId"
        class="flex flex-col gap-3 rounded-lg border border-white/8 bg-white/5 p-4"
      >
        <div class="flex flex-wrap items-center gap-2">
          <p class="min-w-0 flex-1 truncate text-sm font-medium text-white">
            {{ job.label }}
          </p>
          <UBadge
            color="neutral"
            variant="soft"
            :label="ui.modes[job.mode] ?? job.mode"
          />
          <UBadge
            color="neutral"
            variant="soft"
            :label="ui.media[detectKind(job.inputPath)] ?? detectKind(job.inputPath)"
          />
          <UBadge
            v-if="mixedJobProgress(job)"
            :color="statusColor(mixedJobProgress(job)?.status)"
            variant="soft"
            :label="ui.status[mixedJobProgress(job)?.status || 'queued'] ?? mixedJobProgress(job)?.status ?? ui.status.queued"
          />
        </div>
        <p class="truncate text-xs text-stone-500">
          {{ job.inputPath }}
        </p>
        <p class="text-xs text-stone-400">
          {{ describeActivity(job) }}
        </p>
        <div
          v-if="mixedJobProgress(job)"
          class="space-y-2"
        >
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
            @click="emit('removeActivityJob', job.jobId)"
          >
            {{ ui.activity.remove }}
          </UButton>
        </div>
      </div>
    </div>

    <UEmptyState
      v-else
      icon="i-lucide-layers-3"
      :title="ui.activity.emptyTitle"
      :description="ui.activity.emptyDescription"
    />
  </UCard>
</template>
