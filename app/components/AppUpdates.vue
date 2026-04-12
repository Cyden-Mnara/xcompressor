<script setup lang="ts">
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
</script>

<template>
  <UCard :ui="{ root: 'border border-white/10 bg-stone-950/85 ring-0' }">
    <template #header>
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-emerald-300">
            App updates
          </p>
          <p class="mt-2 text-sm leading-6 text-stone-300">
            {{ updateLoading ? 'Checking GitHub release channel...' : (updateStatus?.message || 'Update status unavailable.') }}
          </p>
        </div>
        <UBadge
          v-if="updateStatus"
          :color="updateStatus.updateReady ? 'warning' : (updateStatus.configured ? 'success' : 'neutral')"
          variant="soft"
          :label="updateStatus.updateReady ? 'update ready' : (updateStatus.configured ? 'current' : 'not configured')"
        />
      </div>
    </template>

    <div class="space-y-3">
      <div class="grid grid-cols-2 gap-3">
        <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
          <p class="text-xs text-stone-500">
            Installed
          </p>
          <p class="mt-1 text-lg font-semibold text-white">
            {{ updateStatus?.currentVersion || bootstrap?.version || 'n/a' }}
          </p>
        </div>
        <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
          <p class="text-xs text-stone-500">
            Available
          </p>
          <p class="mt-1 text-lg font-semibold text-white">
            {{ updateStatus?.availableVersion || 'latest' }}
          </p>
        </div>
      </div>

      <p
        v-if="updateStatus?.pubDate"
        class="text-xs leading-6 text-stone-400"
      >
        Release date: {{ formatUpdateDate(updateStatus.pubDate) }}
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
          Check now
        </UButton>
        <UButton
          v-if="updateStatus?.updateReady"
          color="success"
          icon="i-lucide-download"
          :loading="updateInstalling"
          @click="emit('installUpdate')"
        >
          Install update
        </UButton>
      </div>
    </div>
  </UCard>
</template>
