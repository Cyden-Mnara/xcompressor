<script setup lang="ts">
type BootstrapPreset = {
  id: string
  label: string
  description: string
  qualityRange: string
  sizeReductionRange: string
}

type BootstrapCapability = {
  kind: string
  compressionModes: string[]
  conversions: string[]
}

type BootstrapData = {
  version: string
  presets: BootstrapPreset[]
  mediaCapabilities: BootstrapCapability[]
  gifWorkflow: string[]
}

defineProps<{
  bootstrap: BootstrapData | null
  activePreset: BootstrapPreset | undefined
  activeMediaType: string
}>()

const emit = defineEmits<{
  selectMediaType: [mediaType: string]
}>()

const mediaTypes = [
  { label: 'Video', value: 'video', icon: 'i-lucide-video' },
  { label: 'Image', value: 'image', icon: 'i-lucide-image' },
  { label: 'Audio', value: 'audio', icon: 'i-lucide-music' }
]
</script>

<template>
  <UCard
    :ui="{ root: 'thin-scrollbar h-full overflow-y-auto border border-white/10 bg-stone-950/85 ring-0 lg:max-h-[calc(100dvh-5rem)]' }"
  >
    <template #header>
      <div class="space-y-3">
        <div class="flex flex-wrap items-center gap-2">
          <UBadge
            color="primary"
            variant="soft"
            label="Workspace overview"
          />
          <UBadge
            v-if="bootstrap?.version"
            color="neutral"
            variant="soft"
            :label="`v${bootstrap.version}`"
          />
        </div>
        <div>
          <h1 class="text-3xl font-semibold tracking-tight text-white">
            xcompressor
          </h1>
          <p class="mt-2 text-sm leading-6 text-stone-300">
            Choose what you are working on, add media, tune the output, then run the queue.
          </p>
        </div>
      </div>
    </template>

    <div class="space-y-4">
      <div class="grid gap-2 sm:grid-cols-3 xl:grid-cols-1">
        <UButton
          v-for="mediaType in mediaTypes"
          :key="mediaType.value"
          :icon="mediaType.icon"
          :color="activeMediaType === mediaType.value ? 'primary' : 'neutral'"
          :variant="activeMediaType === mediaType.value ? 'solid' : 'soft'"
          size="lg"
          block
          @click="emit('selectMediaType', mediaType.value)"
        >
          {{ mediaType.label }}
        </UButton>
      </div>

      <div class="space-y-4 grid gap-4 grid-cols-2 xl:grid-cols-1">
        <div class="rounded-lg border border-amber-500/20 bg-amber-500/8 p-4 h-full xl:h-max">
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-amber-300">
            Active preset
          </p>
          <p class="mt-2 text-xl font-medium text-white">
            {{ activePreset?.label || 'Balanced' }}
          </p>
          <p class="mt-2 text-sm leading-6 text-stone-300">
            {{ activePreset?.description }}
          </p>
          <div class="mt-3 grid grid-cols-2 gap-3 text-sm">
            <div>
              <p class="text-stone-500">
                Quality
              </p>
              <p class="font-medium text-stone-100">
                {{ activePreset?.qualityRange }}
              </p>
            </div>
            <div>
              <p class="text-stone-500">
                Size delta
              </p>
              <p class="font-medium text-stone-100">
                {{ activePreset?.sizeReductionRange }}
              </p>
            </div>
          </div>
        </div>

        <div class="space-y-3">
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
            Workflow
          </p>
          <ol class="space-y-2">
            <li
              v-for="(step, index) in bootstrap?.gifWorkflow || []"
              :key="step"
              class="flex gap-3 rounded-lg border border-white/8 bg-white/5 p-3"
            >
              <div
                class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-amber-500/15 text-xs font-semibold text-amber-300"
              >
                {{ index + 1 }}
              </div>
              <p class="text-xs leading-6 text-stone-300">
                {{ step }}
              </p>
            </li>
          </ol>
        </div>
      </div>
    </div>
  </UCard>
</template>
