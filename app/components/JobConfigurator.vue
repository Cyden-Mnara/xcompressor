<script setup lang="ts">
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
}>()

const emit = defineEmits<{
  pickFiles: []
  pickOutputDir: []
  clearQueue: []
  addCurrentActivity: []
  runBatch: []
  cancelBatch: []
  updateSelectedJob: [patch: Partial<MixedJob>]
}>()

const outputDir = defineModel<string>('outputDir', { required: true })
const mode = defineModel<string>('mode', { required: true })

const presetItems = computed(() => (props.bootstrap?.presets || []).map(preset => ({ label: preset.label, value: preset.id })))
const currentActivityDisabled = computed(() => !outputDir.value || (!props.activityQueueCount && (mode.value === 'gif' ? !props.gifQueueCount : !props.filesCount)))
const runLabel = computed(() => {
  if (props.processing) {
    return 'Processing batch...'
  }

  if (props.activityQueueCount) {
    return 'Run mixed activity batch'
  }

  return mode.value === 'gif' ? 'Generate GIF batch' : 'Run batch'
})
const selectedModeOptions = computed(() => props.selectedMediaKind === 'video'
  ? [
      { label: 'Compress', value: 'compress' },
      { label: 'Convert', value: 'convert' },
      { label: 'Create GIF', value: 'gif' }
    ]
  : [
      { label: 'Compress', value: 'compress' },
      { label: 'Convert', value: 'convert' }
    ])
const selectedTargetLabel = computed(() => `${props.selectedMediaKind.charAt(0).toUpperCase()}${props.selectedMediaKind.slice(1)} target`)
const selectedTargetItems = computed(() => {
  if (props.selectedMediaKind === 'image') {
    return props.imageTargets
  }

  if (props.selectedMediaKind === 'audio') {
    return props.audioTargets
  }

  return props.videoTargets
})
const selectedTarget = computed({
  get() {
    if (!props.selectedJob) {
      return ''
    }

    if (props.selectedMediaKind === 'image') {
      return props.selectedJob.imageFormat
    }

    if (props.selectedMediaKind === 'audio') {
      return props.selectedJob.audioFormat
    }

    return props.selectedJob.videoFormat
  },
  set(value: string) {
    if (props.selectedMediaKind === 'image') {
      emit('updateSelectedJob', { imageFormat: value })
      return
    }

    if (props.selectedMediaKind === 'audio') {
      emit('updateSelectedJob', { audioFormat: value })
      return
    }

    emit('updateSelectedJob', { videoFormat: value })
  }
})

function basename(path: string) {
  return path.split(/[\\/]/).pop() || path
}
</script>

<template>
  <UCard :ui="{ root: 'thin-scrollbar overflow-y-auto border border-white/10 bg-stone-950/85 ring-0 lg:max-h-[calc(100dvh-5rem)]' }">
    <template #header>
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
            Configure
          </p>
          <h2 class="mt-2 text-2xl font-semibold text-white">
            {{ selectedMediaKind || 'Media' }} setup
          </h2>
        </div>
        <UBadge
          color="primary"
          variant="soft"
          :label="filesCount + ' source files'"
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
          Add media
        </UButton>
        <UInput
          v-model="outputDir"
          icon="i-lucide-folder-open"
          size="lg"
          placeholder="Choose an output directory"
        />
        <UButton
          icon="i-lucide-folder-output"
          color="neutral"
          variant="soft"
          size="lg"
          class="justify-center"
          @click="emit('pickOutputDir')"
        >
          Output
        </UButton>
        <UButton
          icon="i-lucide-trash-2"
          color="error"
          variant="ghost"
          size="lg"
          class="justify-center"
          @click="emit('clearQueue')"
        >
          Clear
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
          Apply defaults to queue
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
        {{ cancelPending ? 'Stopping batch...' : 'Cancel running batch' }}
      </UButton>

      <p
        v-if="mode === 'gif' && !gifQueueCount"
        class="text-sm leading-6 text-amber-300"
      >
        Add at least one GIF clip before running export.
      </p>
      <p
        v-if="activityQueueCount"
        class="text-sm leading-6 text-amber-300"
      >
        Each queued file can keep these defaults or use its own settings.
      </p>

      <div
        v-if="selectedJob"
        class="space-y-4 rounded-lg border border-amber-400/25 bg-amber-400/8 p-4"
      >
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="min-w-0">
            <p class="text-xs font-semibold uppercase tracking-[0.2em] text-amber-300">
              Selected file settings
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
          <UFormField label="Mode">
            <USelect
              :model-value="selectedJob.mode"
              :items="selectedModeOptions"
              option-attribute="label"
              value-attribute="value"
              @update:model-value="emit('updateSelectedJob', { mode: String($event) })"
            />
          </UFormField>

          <UFormField label="Preset">
            <USelect
              :model-value="selectedJob.presetId"
              :items="presetItems"
              option-attribute="label"
              value-attribute="value"
              @update:model-value="emit('updateSelectedJob', { presetId: String($event) })"
            />
          </UFormField>

          <UFormField :label="selectedTargetLabel">
            <USelect
              v-model="selectedTarget"
              :items="selectedTargetItems"
            />
          </UFormField>

          <UFormField
            v-if="selectedJob.mode !== 'gif' && selectedMediaKind !== 'audio'"
            label="Resize long edge"
          >
            <UInputNumber
              :model-value="selectedJob.resizeLongEdge"
              :min="320"
              :max="4096"
              :step="10"
              @update:model-value="emit('updateSelectedJob', { resizeLongEdge: Number($event) })"
            />
          </UFormField>

          <UFormField label="Output directory">
            <UInput
              :model-value="selectedJob.outputDir"
              icon="i-lucide-folder-open"
              placeholder="Choose an output directory"
              @update:model-value="emit('updateSelectedJob', { outputDir: String($event) })"
            />
          </UFormField>
        </div>

        <div
          v-if="selectedJob.mode === 'gif' && selectedJob.gif"
          class="grid gap-4 lg:grid-cols-4"
        >
          <UFormField label="Start">
            <UInputNumber
              :model-value="selectedJob.gif.startSeconds"
              :min="0"
              :step="0.5"
              @update:model-value="emit('updateSelectedJob', { gif: { ...selectedJob.gif!, startSeconds: Number($event) } })"
            />
          </UFormField>
          <UFormField label="Duration">
            <UInputNumber
              :model-value="selectedJob.gif.durationSeconds"
              :min="0.5"
              :step="0.5"
              @update:model-value="emit('updateSelectedJob', { gif: { ...selectedJob.gif!, durationSeconds: Number($event) } })"
            />
          </UFormField>
          <UFormField label="FPS">
            <UInputNumber
              :model-value="selectedJob.gif.fps"
              :min="1"
              :max="30"
              @update:model-value="emit('updateSelectedJob', { gif: { ...selectedJob.gif!, fps: Number($event) } })"
            />
          </UFormField>
          <UFormField label="Width">
            <UInputNumber
              :model-value="selectedJob.gif.width"
              :min="160"
              :max="1280"
              :step="20"
              @update:model-value="emit('updateSelectedJob', { gif: { ...selectedJob.gif!, width: Number($event) } })"
            />
          </UFormField>
        </div>
      </div>
      <div
        v-else-if="activityQueueCount"
        class="rounded-lg border border-white/10 bg-white/5 p-4 text-sm leading-6 text-stone-300"
      >
        Select a queued file to adjust its mode, format, resize, or output directory.
      </div>
    </div>
  </UCard>
</template>
