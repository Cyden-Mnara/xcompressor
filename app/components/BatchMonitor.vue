<script setup lang="ts">
defineProps<{
  overallProgress: number
  activityQueueCount: number
  gifQueueCount: number
  filesCount: number
  completedJobs: number
  mode: string
  cancelPending: boolean
}>()
</script>

<template>
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
        <span v-if="activityQueueCount">
          {{ completedJobs }}/{{ activityQueueCount || 0 }} mixed activity jobs finished.
        </span>
        <span v-else-if="mode === 'gif'">
          {{ completedJobs }}/{{ gifQueueCount || 0 }} queued GIF clips finished.
        </span>
        <span v-else>
          {{ completedJobs }}/{{ filesCount || 0 }} jobs finished.
        </span>
        <span
          v-if="cancelPending"
          class="text-amber-300"
        >
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
</template>
