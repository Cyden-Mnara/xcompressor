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
  mediaType: string
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
const resizeLongEdge = defineModel<number | null>('resizeLongEdge', { required: true })

const presetItems = computed(() => (props.bootstrap?.presets || []).map(preset => ({ label: preset.label, value: preset.id })))
const currentActivityDisabled = computed(() => !outputDir.value || (mode.value === 'gif' ? !props.gifQueueCount : !props.filesCount))
const mediaTargetLabel = computed(() => `${props.mediaType.charAt(0).toUpperCase()}${props.mediaType.slice(1)} target`)
const mediaTarget = computed({
  get() {
    if (props.mediaType === 'image') {
      return imageFormat.value
    }

    if (props.mediaType === 'audio') {
      return audioFormat.value
    }

    return videoFormat.value
  },
  set(value: string) {
    if (props.mediaType === 'image') {
      imageFormat.value = value
      return
    }

    if (props.mediaType === 'audio') {
      audioFormat.value = value
      return
    }

    videoFormat.value = value
  }
})
const mediaTargetItems = computed(() => {
  if (props.mediaType === 'image') {
    return props.imageTargets
  }

  if (props.mediaType === 'audio') {
    return props.audioTargets
  }

  return props.videoTargets
})
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
  <UCard :ui="{ root: 'thin-scrollbar overflow-y-auto border border-white/10 bg-stone-950/85 ring-0 lg:max-h-[calc(100dvh-5rem)]' }">
    <template #header>
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
            Configure
          </p>
          <h2 class="mt-2 text-2xl font-semibold text-white">
            {{ mediaType }} setup
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

        <UFormField :label="mediaTargetLabel">
          <USelect
            v-model="mediaTarget"
            :items="mediaTargetItems"
          />
        </UFormField>

        <UFormField
          v-if="mode !== 'gif' && mediaType !== 'audio'"
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
        The saved activity queue is what runs.
      </p>
    </div>
  </UCard>
</template>
