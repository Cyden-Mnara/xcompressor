<script setup lang="ts">
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

  const target = detectKind(job.inputPath) === 'video'
    ? job.videoFormat
    : detectKind(job.inputPath) === 'image'
      ? job.imageFormat
      : job.audioFormat
  const resize = job.resizeLongEdge ? ` • ${job.resizeLongEdge}px edge` : ''
  return `${job.mode} -> ${target}${resize}`
}
</script>

<template>
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
          <UBadge
            color="primary"
            variant="soft"
            :label="activityQueue.length + ' queued'"
          />
          <UButton
            icon="i-lucide-trash-2"
            color="error"
            variant="ghost"
            :disabled="!activityQueue.length"
            @click="emit('clearActivityQueue')"
          >
            Clear
          </UButton>
        </div>
      </div>
    </template>

    <div
      v-if="activityQueue.length"
      class="max-h-[32rem] space-y-3 overflow-y-auto pr-1"
    >
      <div
        v-for="job in activityQueue"
        :key="job.jobId"
        class="flex flex-col gap-3 rounded-2xl border border-white/8 bg-white/5 p-4"
      >
        <div class="flex flex-wrap items-center gap-2">
          <p class="truncate text-sm font-medium text-white">
            {{ job.label }}
          </p>
          <UBadge
            color="neutral"
            variant="soft"
            :label="job.mode"
          />
          <UBadge
            color="neutral"
            variant="soft"
            :label="detectKind(job.inputPath)"
          />
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
</template>
