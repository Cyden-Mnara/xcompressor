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
const appUi = inject('appUi') as AppUiInjection
const ui = computed(() => appUi.value)

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
      { label: appUi.value.modes.gif, value: 'gif' }
    ]
  : [
      { label: appUi.value.modes.compress, value: 'compress' },
      { label: appUi.value.modes.convert, value: 'convert' }
    ])
const selectedTargetLabel = computed(() => `${appUi.value.media[props.selectedMediaKind] ?? appUi.value.media.media} ${appUi.value.job.target}`)
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
              @update:model-value="emit('updateSelectedJob', { mode: String($event) })"
            />
          </UFormField>

          <UFormField :label="ui.job.preset">
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
          class="grid gap-4 lg:grid-cols-4"
        >
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
