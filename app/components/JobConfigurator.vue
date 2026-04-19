<script setup lang="ts">
import type { AppUiInjection } from '~/utils/app-ui'

type BootstrapPreset = {
  id: string
  label: string
}

type BootstrapData = {
  presets: BootstrapPreset[]
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

const props = defineProps<{
  bootstrap: BootstrapData | null
  filesCount: number
  videoTargets: string[]
  imageTargets: string[]
  audioTargets: string[]
  gifQueueCount: number
  activityQueueCount: number
  processing: boolean
  cancelPending: boolean
  canRun: boolean
  selectedJob: MixedJob | null
  selectedMediaKind: string
  selectedGifVideoSrc: string
  selectedGifVideoDuration: number
  gifPreviewError: string
}>()

const emit = defineEmits<{
  pickFiles: []
  pickOutputDir: []
  clearQueue: []
  addCurrentActivity: []
  runBatch: []
  cancelBatch: []
  updateSelectedJob: [patch: Partial<MixedJob>]
  addSelectedGifJob: []
  loadedGifMetadata: [event: Event]
  previewGifError: []
  openSelectedGifExternal: [path: string]
}>()

const outputDir = defineModel<string>('outputDir', { required: true })
const mode = defineModel<string>('mode', { required: true })
const appUi = inject('appUi') as AppUiInjection
const ui = computed(() => appUi.value)
const selectedGifPreviewVideo = ref<HTMLVideoElement | null>(null)

const presetItems = computed(() => (props.bootstrap?.presets || []).map(preset => ({
  label: appUi.value.presets[`${toPresetCopyKey(preset.id)}Label`] ?? preset.label,
  value: preset.id
})))
const currentActivityDisabled = computed(() => !outputDir.value || (!props.activityQueueCount && (mode.value === 'gif' ? !props.gifQueueCount : !props.filesCount)))
const runLabel = computed(() => {
  if (props.processing) {
    return appUi.value.job.processingBatch
  }

  if (props.activityQueueCount) {
    return appUi.value.job.runMixed
  }

  return mode.value === 'gif' ? appUi.value.job.generateGif : appUi.value.job.runBatch
})
const selectedModeOptions = computed(() => props.selectedMediaKind === 'video'
  ? [
      { label: appUi.value.modes.compress, value: 'compress' },
      { label: appUi.value.modes.convert, value: 'convert' },
      { label: appUi.value.modes['extract-audio'], value: 'extract-audio' },
      { label: appUi.value.modes.gif, value: 'gif' }
    ]
  : [
      { label: appUi.value.modes.compress, value: 'compress' },
      { label: appUi.value.modes.convert, value: 'convert' }
    ])
const selectedTargetKind = computed(() => props.selectedMediaKind === 'video' && props.selectedJob?.mode === 'extract-audio'
  ? 'audio'
  : props.selectedMediaKind)
const selectedTargetLabel = computed(() => `${appUi.value.media[selectedTargetKind.value] ?? appUi.value.media.media} ${appUi.value.job.target}`)
const selectedTargetItems = computed(() => {
  if (selectedTargetKind.value === 'image') {
    return props.imageTargets
  }

  if (selectedTargetKind.value === 'audio') {
    return props.audioTargets
  }

  return props.videoTargets
})
const selectedTarget = computed({
  get() {
    if (!props.selectedJob) {
      return ''
    }

    if (selectedTargetKind.value === 'image') {
      return props.selectedJob.imageFormat
    }

    if (selectedTargetKind.value === 'audio') {
      return props.selectedJob.audioFormat
    }

    return props.selectedJob.videoFormat
  },
  set(value: string) {
    if (selectedTargetKind.value === 'image') {
      emit('updateSelectedJob', { imageFormat: value })
      return
    }

    if (selectedTargetKind.value === 'audio') {
      emit('updateSelectedJob', { audioFormat: value })
      return
    }

    emit('updateSelectedJob', { videoFormat: value })
  }
})

function selectValue(value: unknown) {
  if (typeof value === 'string') {
    return value
  }

  if (value && typeof value === 'object' && 'value' in value) {
    return String((value as { value: unknown }).value)
  }

  return String(value)
}

function onSelectedGifLoaded(event: Event) {
  selectedGifPreviewVideo.value = event.target as HTMLVideoElement
  emit('loadedGifMetadata', event)
}

function setSelectedGifStartFromPreview(event: Event) {
  if (!props.selectedJob?.gif) {
    return
  }

  const video = event.target as HTMLVideoElement
  emit('updateSelectedJob', {
    gif: {
      ...props.selectedJob.gif,
      startSeconds: Number(video.currentTime.toFixed(2))
    }
  })
}

function setSelectedGifEndFromPreview(event: Event) {
  if (!props.selectedJob?.gif) {
    return
  }

  const video = event.target as HTMLVideoElement
  const endSeconds = Number(video.currentTime.toFixed(2))
  if (endSeconds <= props.selectedJob.gif.startSeconds + 0.1) {
    return
  }

  emit('updateSelectedJob', {
    gif: {
      ...props.selectedJob.gif,
      durationSeconds: Number((endSeconds - props.selectedJob.gif.startSeconds).toFixed(2))
    }
  })
}

function jumpSelectedGifPreviewToStart() {
  if (selectedGifPreviewVideo.value && props.selectedJob?.gif) {
    selectedGifPreviewVideo.value.currentTime = props.selectedJob.gif.startSeconds
  }
}

function basename(path: string) {
  return path.split(/[\\/]/).pop() || path
}

function toPresetCopyKey(id: string) {
  return id.replace(/-([a-z])/g, (_, letter: string) => letter.toUpperCase())
}
</script>

<template>
  <UCard :ui="{ root: 'thin-scrollbar overflow-y-auto border border-white/10 bg-stone-950/85 ring-0 lg:max-h-[calc(100dvh-5rem)]' }">
    <template #header>
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
            {{ ui.job.configure }}
          </p>
          <h2 class="mt-2 text-2xl font-semibold text-white">
            {{ ui.media[selectedMediaKind] || ui.media.media }} {{ ui.job.setup }}
          </h2>
        </div>
        <UBadge
          color="primary"
          variant="soft"
          :label="`${filesCount} ${ui.job.sourceFiles}`"
        />
      </div>
    </template>

    <div class="space-y-5">
      <div class="grid gap-3 grid-cols-4 lg:grid-cols-[auto_minmax(0,1fr)_auto_auto]">
        <UButton
          icon="i-lucide-folder-plus"
          color="primary"
          size="lg"
          class="justify-center"
          @click="emit('pickFiles')"
        >
          {{ ui.job.addMedia }}
        </UButton>
        <UInput
          v-model="outputDir"
          icon="i-lucide-folder-open"
          size="lg"
          :placeholder="ui.job.outputPlaceholder"
        />
        <UButton
          icon="i-lucide-folder-output"
          color="neutral"
          variant="soft"
          size="lg"
          class="justify-center"
          @click="emit('pickOutputDir')"
        >
          {{ ui.job.output }}
        </UButton>
        <UButton
          icon="i-lucide-trash-2"
          color="error"
          variant="ghost"
          size="lg"
          class="justify-center"
          @click="emit('clearQueue')"
        >
          {{ ui.job.clear }}
        </UButton>
      </div>

      <div class="grid gap-3 md:grid-cols-2">
        <UButton
          block
          size="xl"
          color="neutral"
          variant="soft"
          icon="i-lucide-plus"
          :disabled="currentActivityDisabled"
          @click="emit('addCurrentActivity')"
        >
          {{ ui.job.applyDefaults }}
        </UButton>
        <UButton
          block
          size="xl"
          color="primary"
          icon="i-lucide-play"
          :loading="processing"
          :disabled="!canRun"
          @click="emit('runBatch')"
        >
          {{ runLabel }}
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
        @click="emit('cancelBatch')"
      >
        {{ cancelPending ? ui.job.stoppingBatch : ui.job.cancelRunning }}
      </UButton>

      <p
        v-if="mode === 'gif' && !gifQueueCount"
        class="text-sm leading-6 text-amber-300"
      >
        {{ ui.job.gifClipRequired }}
      </p>
      <p
        v-if="activityQueueCount"
        class="text-sm leading-6 text-amber-300"
      >
        {{ ui.job.activityDefaults }}
      </p>

      <div
        v-if="selectedJob"
        class="space-y-4 rounded-lg border border-amber-400/25 bg-amber-400/8 p-4"
      >
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="min-w-0">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-amber-300">
              {{ ui.job.selectedSettings }}
            </p>
            <p class="mt-2 truncate text-lg font-semibold text-white">
              {{ basename(selectedJob.inputPath) }}
            </p>
            <p class="mt-1 truncate text-xs text-stone-400">
              {{ selectedJob.inputPath }}
            </p>
          </div>
          <UBadge
            color="neutral"
            variant="soft"
            :label="selectedMediaKind"
          />
        </div>

        <div class="grid gap-4 lg:grid-cols-2">
          <UFormField :label="ui.job.mode">
            <USelect
              :model-value="selectedJob.mode"
              :items="selectedModeOptions"
              option-attribute="label"
              value-attribute="value"
              @update:model-value="emit('updateSelectedJob', { mode: selectValue($event) })"
            />
          </UFormField>

          <UFormField :label="ui.job.preset">
            <USelect
              :model-value="selectedJob.presetId"
              :items="presetItems"
              option-attribute="label"
              value-attribute="value"
              @update:model-value="emit('updateSelectedJob', { presetId: selectValue($event) })"
            />
          </UFormField>

          <UFormField :label="selectedTargetLabel">
            <USelect
              v-model="selectedTarget"
              :items="selectedTargetItems"
            />
          </UFormField>

          <UFormField
            v-if="selectedJob.mode !== 'gif' && selectedJob.mode !== 'extract-audio' && selectedMediaKind !== 'audio'"
            :label="ui.job.resizeLongEdge"
          >
            <UInputNumber
              :model-value="selectedJob.resizeLongEdge"
              :min="320"
              :max="4096"
              :step="10"
              @update:model-value="emit('updateSelectedJob', { resizeLongEdge: Number($event) })"
            />
          </UFormField>

          <UFormField :label="ui.job.outputDirectory">
            <UInput
              :model-value="selectedJob.outputDir"
              icon="i-lucide-folder-open"
              :placeholder="ui.job.outputPlaceholder"
              @update:model-value="emit('updateSelectedJob', { outputDir: String($event) })"
            />
          </UFormField>
        </div>

        <div
          v-if="selectedJob.mode === 'gif' && selectedJob.gif"
          class="space-y-4"
        >
          <div
            v-if="selectedGifVideoSrc"
            class="space-y-3 rounded-lg border border-white/10 bg-black/20 p-3"
          >
            <video
              class="block aspect-video max-w-full rounded-lg border border-white/10 bg-black object-contain"
              :src="selectedGifVideoSrc"
              controls
              preload="metadata"
              @loadedmetadata="onSelectedGifLoaded"
              @play="setSelectedGifStartFromPreview"
              @pause="setSelectedGifEndFromPreview"
              @error="emit('previewGifError')"
            />
            <div
              v-if="gifPreviewError"
              class="rounded-lg border border-amber-500/20 bg-amber-500/8 p-3"
            >
              <p class="text-sm leading-6 text-amber-200">
                {{ gifPreviewError }}
              </p>
            </div>
            <div class="flex flex-wrap items-center gap-2">
              <UButton
                color="neutral"
                variant="soft"
                icon="i-lucide-play"
                :disabled="Boolean(gifPreviewError) || !selectedGifPreviewVideo"
                @click="selectedGifPreviewVideo?.play()"
              >
                {{ ui.gif.playPreview }}
              </UButton>
              <UButton
                color="neutral"
                variant="soft"
                icon="i-lucide-pause"
                :disabled="Boolean(gifPreviewError) || !selectedGifPreviewVideo"
                @click="selectedGifPreviewVideo?.pause()"
              >
                {{ ui.gif.pausePreview }}
              </UButton>
              <UButton
                color="neutral"
                variant="soft"
                icon="i-lucide-skip-back"
                :disabled="Boolean(gifPreviewError) || !selectedGifPreviewVideo"
                @click="jumpSelectedGifPreviewToStart"
              >
                {{ ui.gif.jumpStart }}
              </UButton>
              <UButton
                color="neutral"
                variant="soft"
                icon="i-lucide-monitor-play"
                @click="emit('openSelectedGifExternal', selectedJob.inputPath)"
              >
                {{ ui.gif.openExternal }}
              </UButton>
              <p class="text-xs leading-6 text-stone-400">
                {{ ui.gif.videoLength }}: {{ selectedGifVideoDuration ? `${selectedGifVideoDuration.toFixed(1)}s` : ui.gif.loading }}
              </p>
            </div>
          </div>

          <div class="grid gap-4 lg:grid-cols-4">
            <UFormField :label="ui.job.start">
              <UInputNumber
                :model-value="selectedJob.gif.startSeconds"
                :min="0"
                :step="0.5"
                @update:model-value="emit('updateSelectedJob', { gif: { ...selectedJob.gif!, startSeconds: Number($event) } })"
              />
            </UFormField>
            <UFormField :label="ui.job.duration">
              <UInputNumber
                :model-value="selectedJob.gif.durationSeconds"
                :min="0.5"
                :step="0.5"
                @update:model-value="emit('updateSelectedJob', { gif: { ...selectedJob.gif!, durationSeconds: Number($event) } })"
              />
            </UFormField>
            <UFormField :label="ui.job.fps">
              <UInputNumber
                :model-value="selectedJob.gif.fps"
                :min="1"
                :max="30"
                @update:model-value="emit('updateSelectedJob', { gif: { ...selectedJob.gif!, fps: Number($event) } })"
              />
            </UFormField>
            <UFormField :label="ui.job.width">
              <UInputNumber
                :model-value="selectedJob.gif.width"
                :min="160"
                :max="1280"
                :step="20"
                @update:model-value="emit('updateSelectedJob', { gif: { ...selectedJob.gif!, width: Number($event) } })"
              />
            </UFormField>
          </div>

          <UButton
            block
            color="primary"
            icon="i-lucide-plus"
            @click="emit('addSelectedGifJob')"
          >
            {{ ui.job.addGifJob }}
          </UButton>
        </div>
      </div>
      <div
        v-else-if="activityQueueCount"
        class="rounded-lg border border-white/10 bg-white/5 p-4 text-sm leading-6 text-stone-300"
      >
        {{ ui.job.selectQueued }}
      </div>
    </div>
  </UCard>
</template>
