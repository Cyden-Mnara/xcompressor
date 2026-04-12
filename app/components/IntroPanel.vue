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
}>()
</script>

<template>
  <UCard :ui="{ root: 'h-full border border-white/10 bg-stone-950/85 ring-0' }">
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
            Multimedia compression, conversion, and GIF jobs arranged like a desktop workspace instead of a landing page.
          </p>
        </div>
      </div>
    </template>

    <div class="space-y-4">
      <div class="rounded-2xl border border-amber-500/20 bg-amber-500/8 p-4">
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
          Capabilities
        </p>
        <div
          v-for="capability in bootstrap?.mediaCapabilities || []"
          :key="capability.kind"
          class="rounded-2xl border border-white/8 bg-white/5 p-4"
        >
          <div class="flex items-center justify-between gap-3">
            <h2 class="text-sm font-medium capitalize text-white">
              {{ capability.kind }}
            </h2>
            <UBadge
              color="neutral"
              variant="soft"
              :label="capability.conversions.length + ' targets'"
            />
          </div>
          <p class="mt-2 text-xs leading-6 text-stone-400">
            {{ capability.compressionModes.join(' • ') }}
          </p>
        </div>
      </div>

      <div class="space-y-3">
        <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
          GIF workflow
        </p>
        <ol class="space-y-2">
          <li
            v-for="(step, index) in bootstrap?.gifWorkflow || []"
            :key="step"
            class="flex gap-3 rounded-2xl border border-white/8 bg-white/5 p-3"
          >
            <div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-amber-500/15 text-xs font-semibold text-amber-300">
              {{ index + 1 }}
            </div>
            <p class="text-xs leading-6 text-stone-300">
              {{ step }}
            </p>
          </li>
        </ol>
      </div>
    </div>
  </UCard>
</template>
