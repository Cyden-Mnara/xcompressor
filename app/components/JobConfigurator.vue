<script setup lang="ts">
type BootstrapPreset = {
  id: string
  label: string
}

type BootstrapData = {
  presets: BootstrapPreset[]
}

const props = defineProps<{
  bootstrap: BootstrapData | null
  filesCount: number
  modeOptions: { label: string, value: string }[]
  videoTargets: string[]
  imageTargets: string[]
  audioTargets: string[]
  gifQueueCount: number
  activityQueueCount: number
  processing: boolean
  cancelPending: boolean
  canRun: boolean
}>()

const emit = defineEmits<{
  pickFiles: []
  pickOutputDir: []
  clearQueue: []
  addCurrentActivity: []
  runBatch: []
  cancelBatch: []
}>()

const outputDir = defineModel<string>('outputDir', { required: true })
const mode = defineModel<string>('mode', { required: true })
const presetId = defineModel<string>('presetId', { required: true })
const videoFormat = defineModel<string>('videoFormat', { required: true })
const imageFormat = defineModel<string>('imageFormat', { required: true })
const audioFormat = defineModel<string>('audioFormat', { required: true })
const maxParallelJobs = defineModel<number>('maxParallelJobs', { required: true })
const resizeLongEdge = defineModel<number | null>('resizeLongEdge', { required: true })

const presetItems = computed(() => (props.bootstrap?.presets || []).map(preset => ({ label: preset.label, value: preset.id })))
const currentActivityDisabled = computed(() => !outputDir.value || (mode.value === 'gif' ? !props.gifQueueCount : !props.filesCount))
const runLabel = computed(() => {
  if (props.processing) {
    return 'Processing batch...'
  }

  if (props.activityQueueCount) {
    return 'Run mixed activity batch'
  }

  return mode.value === 'gif' ? 'Generate GIF batch' : 'Run batch'
})
</script>

<template>
  <UCard :ui="{ root: 'border border-white/10 bg-stone-950/85 ring-0' }">
    <template #header>
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
            Configure
          </p>
          <h2 class="mt-2 text-2xl font-semibold text-white">
            Files and operations
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
      <div class="flex flex-wrap gap-3">
        <UButton
          icon="i-lucide-folder-plus"
          color="primary"
          size="lg"
          @click="emit('pickFiles')"
        >
          Add media
        </UButton>
        <UButton
          icon="i-lucide-folder-output"
          color="neutral"
          variant="soft"
          size="lg"
          @click="emit('pickOutputDir')"
        >
          Output directory
        </UButton>
        <UButton
          icon="i-lucide-trash-2"
          color="error"
          variant="ghost"
          size="lg"
          @click="emit('clearQueue')"
        >
          Clear all
        </UButton>
      </div>

      <UInput
        v-model="outputDir"
        icon="i-lucide-folder-open"
        size="xl"
        placeholder="Choose an output directory"
      />

      <div class="grid gap-4 lg:grid-cols-2">
        <UFormField label="Mode">
          <USelect
            v-model="mode"
            :items="modeOptions"
            option-attribute="label"
            value-attribute="value"
          />
        </UFormField>

        <UFormField label="Preset">
          <USelect
            v-model="presetId"
            :items="presetItems"
            option-attribute="label"
            value-attribute="value"
          />
        </UFormField>

        <UFormField label="Video target">
          <USelect
            v-model="videoFormat"
            :items="videoTargets"
          />
        </UFormField>

        <UFormField label="Image target">
          <USelect
            v-model="imageFormat"
            :items="imageTargets"
          />
        </UFormField>

        <UFormField label="Audio target">
          <USelect
            v-model="audioFormat"
            :items="audioTargets"
          />
        </UFormField>

        <UFormField label="Parallel jobs">
          <UInputNumber
            v-model="maxParallelJobs"
            :min="1"
            :max="8"
          />
        </UFormField>

        <UFormField
          v-if="mode !== 'gif'"
          label="Resize long edge"
        >
          <UInputNumber
            v-model="resizeLongEdge"
            :min="320"
            :max="4096"
            :step="10"
          />
        </UFormField>
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
          Add current activity
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
        class="text-sm leading-6 text-sky-300"
      >
        Saved activities override the current editor when you run the batch.
      </p>
    </div>
  </UCard>
</template>
