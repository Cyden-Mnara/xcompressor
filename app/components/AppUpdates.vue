<script setup lang="ts">
import type { AppUiInjection } from '~/utils/app-ui'

type AppUpdateStatus = {
  configured: boolean
  currentVersion: string
  availableVersion: string | null
  notes: string | null
  pubDate: string | null
  updateReady: boolean
  message: string
}

type BootstrapData = {
  version: string
}

defineProps<{
  updateStatus: AppUpdateStatus | null
  bootstrap: BootstrapData | null
  updateLoading: boolean
  updateInstalling: boolean
  formatUpdateDate: (value: string | null) => string
}>()

const emit = defineEmits<{
  checkForUpdates: []
  installUpdate: []
}>()
const appUi = inject('appUi') as AppUiInjection
const ui = computed(() => appUi.value)
</script>

<template>
  <UCard :ui="{ root: 'border border-white/10 bg-stone-950/85 ring-0' }">
    <template #header>
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-emerald-300">
            {{ ui.updates.title }}
          </p>
          <p class="mt-2 text-sm leading-6 text-stone-300">
            {{ updateLoading ? ui.updates.checking : (updateStatus?.message || ui.updates.unavailable) }}
          </p>
        </div>
        <UBadge
          v-if="updateStatus"
          :color="updateStatus.updateReady ? 'warning' : (updateStatus.configured ? 'success' : 'neutral')"
          variant="soft"
          :label="updateStatus.updateReady ? ui.updates.ready : (updateStatus.configured ? ui.updates.current : ui.updates.notConfigured)"
        />
      </div>
    </template>

    <div class="space-y-3">
      <div class="grid grid-cols-2 gap-3">
        <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
          <p class="text-xs text-stone-500">
            {{ ui.updates.installed }}
          </p>
          <p class="mt-1 text-lg font-semibold text-white">
            {{ updateStatus?.currentVersion || bootstrap?.version || 'n/a' }}
          </p>
        </div>
        <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
          <p class="text-xs text-stone-500">
            {{ ui.updates.available }}
          </p>
          <p class="mt-1 text-lg font-semibold text-white">
            {{ updateStatus?.availableVersion || ui.updates.latest }}
          </p>
        </div>
      </div>

      <p
        v-if="updateStatus?.pubDate"
        class="text-xs leading-6 text-stone-400"
      >
        {{ ui.updates.releaseDate }}: {{ formatUpdateDate(updateStatus.pubDate) }}
      </p>
      <p
        v-if="updateStatus?.notes"
        class="text-sm leading-6 text-stone-300"
      >
        {{ updateStatus.notes }}
      </p>

      <div class="flex flex-wrap gap-3">
        <UButton
          color="neutral"
          variant="soft"
          icon="i-lucide-refresh-ccw"
          :loading="updateLoading"
          @click="emit('checkForUpdates')"
        >
          {{ ui.updates.checkNow }}
        </UButton>
        <UButton
          v-if="updateStatus?.updateReady"
          color="success"
          icon="i-lucide-download"
          :loading="updateInstalling"
          @click="emit('installUpdate')"
        >
          {{ ui.updates.install }}
        </UButton>
      </div>
    </div>
  </UCard>
</template>
