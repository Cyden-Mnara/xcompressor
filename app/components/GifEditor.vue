<script setup lang="ts">
type GifOptions = {
  startSeconds: number
  durationSeconds: number
  fps: number
  width: number
}

defineProps<{
  mode: string
  gifQueueCount: number
  selectedGifVideoSrc: string
  selectedGifVideoDuration: number
  videoFileOptions: { label: string, value: string }[]
  gifPreviewError: string
  gifPreviewVideo: HTMLVideoElement | null
  gifClipRangeLabel: string
  videoFilesCount: number
  nonVideoFilesCount: number
}>()

const emit = defineEmits<{
  loadedMetadata: [event: Event]
  pausePreview: [event: Event]
  previewError: []
  openExternal: []
  usePausedTime: [event: Event]
  jumpToClipStart: []
  setGifStart: [value: number]
  setGifEnd: [value: number]
  addGifSegment: []
}>()

const gifOptions = defineModel<GifOptions>('gifOptions', { required: true })
const gifEndSeconds = defineModel<number>('gifEndSeconds', { required: true })
const selectedGifVideoModel = defineModel<string>('selectedGifVideo', { required: true })

function basename(path: string) {
  return path.split(/[\\/]/).pop() || path
}
</script>

<template>
  <UCard
    v-if="mode === 'gif'"
    :ui="{ root: 'border border-amber-500/20 bg-stone-950/85 ring-0' }"
  >
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
        <UBadge
          color="warning"
          variant="soft"
          :label="gifQueueCount + ' clips queued'"
        />
      </div>
    </template>

    <div class="grid gap-4 xl:grid-cols-[1.2fr_0.8fr]">
      <div class="min-w-0 space-y-4 overflow-hidden rounded-2xl border border-white/10 bg-black/20 p-4">
        <UFormField
          label="Preview source"
          description="Choose the video you want to clip from."
        >
          <USelect
            v-model="selectedGifVideoModel"
            :items="videoFileOptions"
            option-attribute="label"
            value-attribute="value"
            :ui="{ base: 'min-w-0 w-full', content: 'max-w-full' }"
          />
        </UFormField>

        <div
          v-if="selectedGifVideoSrc"
          class="min-w-0 space-y-3"
        >
          <p class="truncate text-xs text-stone-500">
            {{ basename(selectedGifVideoModel) }}
          </p>
          <video
            class="block aspect-video max-w-full rounded-2xl border border-white/10 bg-black object-contain"
            :src="selectedGifVideoSrc"
            controls
            preload="metadata"
            @loadedmetadata="emit('loadedMetadata', $event)"
            @pause="emit('pausePreview', $event)"
            @error="emit('previewError')"
          />
          <div
            v-if="gifPreviewError"
            class="rounded-2xl border border-amber-500/20 bg-amber-500/8 p-4"
          >
            <p class="text-sm leading-6 text-amber-200">
              {{ gifPreviewError }}
            </p>
            <UButton
              class="mt-3"
              color="warning"
              variant="soft"
              icon="i-lucide-external-link"
              @click="emit('openExternal')"
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
              @click="emit('usePausedTime', $event)"
            >
              Use paused time as start
            </UButton>
            <UButton
              color="neutral"
              variant="soft"
              icon="i-lucide-skip-back"
              :disabled="Boolean(gifPreviewError) || !gifPreviewVideo"
              @click="emit('jumpToClipStart')"
            >
              Jump to clip start
            </UButton>
            <UButton
              color="neutral"
              variant="soft"
              icon="i-lucide-monitor-play"
              @click="emit('openExternal')"
            >
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
                  @input="emit('setGifStart', Number(($event.target as HTMLInputElement).value))"
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
                  @input="emit('setGifEnd', Number(($event.target as HTMLInputElement).value))"
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
            <UFormField
              label="Start second"
              description="Where the clip begins inside the source video."
            >
              <UInputNumber
                v-model="gifOptions.startSeconds"
                :min="0"
                :step="0.5"
              />
            </UFormField>
            <UFormField
              label="End second"
              description="Where the clip stops inside the source video."
            >
              <UInputNumber
                v-model="gifEndSeconds"
                :min="0.5"
                :step="0.5"
              />
            </UFormField>
            <UFormField
              label="GIF FPS"
              description="Higher FPS is smoother but creates a larger file."
            >
              <UInputNumber
                v-model="gifOptions.fps"
                :min="1"
                :max="30"
              />
            </UFormField>
            <UFormField
              label="GIF width"
              description="Output width in pixels. Smaller width exports faster."
            >
              <UInputNumber
                v-model="gifOptions.width"
                :min="160"
                :max="1280"
                :step="20"
              />
            </UFormField>
          </div>

          <UButton
            block
            class="mt-4"
            color="primary"
            icon="i-lucide-plus"
            :disabled="!selectedGifVideoModel"
            @click="emit('addGifSegment')"
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
            Eligible video files: {{ videoFilesCount }}. Queued GIF clips: {{ gifQueueCount }}<span v-if="nonVideoFilesCount">. Non-video files stay in the media queue but cannot create GIF clips.</span>
          </p>
        </div>
      </div>
    </div>
  </UCard>
</template>
