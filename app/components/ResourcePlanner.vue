<script setup lang="ts">
import type { AppUiInjection } from '~/utils/app-ui'

type ResourceJobEstimate = {
  jobId: string
}

type ResourcePlan = {
  safeParallelJobs: number
  estimatedParallelMemoryMb: number
  canRunInParallel: boolean
  shouldUseSequential: boolean
  summary: string
  jobs: ResourceJobEstimate[]
}

type LiveSystemMetrics = {
  cpuUsagePercent: number
  usedMemoryMb: number
  availableMemoryMb: number
}

defineProps<{
  resourcePlan: ResourcePlan | null
  resourcePlanLoading: boolean
  liveSystemMetrics: LiveSystemMetrics | null
  etaCaption: string
  estimatedMinutesLabel: string
  processing: boolean
  cancelPending: boolean
  runQueueCount: number
  effectiveParallelJobs: number
}>()

const emit = defineEmits<{
  cancelBatch: []
  enableSequentialMode: []
}>()
const appUi = inject('appUi') as AppUiInjection
const ui = computed(() => appUi.value)
</script>

<template>
  <UCard :ui="{ root: 'thin-scrollbar overflow-y-auto border border-white/10 bg-stone-950/85 ring-0 lg:max-h-[calc(100dvh-5rem)]' }">
    <template #header>
      <div>
        <p class="text-xs font-semibold uppercase tracking-[0.25em] text-sky-300">
          {{ ui.resource.title }}
        </p>
        <p class="mt-2 text-sm leading-6 text-stone-300">
          {{ resourcePlanLoading ? ui.resource.checking : (resourcePlan?.summary || ui.resource.noJobs) }}
        </p>
      </div>
    </template>

    <div class="space-y-3">
      <div class="grid grid-cols-2 gap-3">
        <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
          <p class="text-xs text-stone-500">
            {{ ui.resource.liveCpu }}
          </p>
          <p class="mt-1 text-lg font-semibold text-white">
            {{ liveSystemMetrics ? `${Math.round(liveSystemMetrics.cpuUsagePercent)}%` : 'n/a' }}
          </p>
        </div>
        <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
          <p class="text-xs text-stone-500">
            {{ ui.resource.liveRam }}
          </p>
          <p class="mt-1 text-lg font-semibold text-white">
            {{ liveSystemMetrics ? `${liveSystemMetrics.usedMemoryMb} MB` : 'n/a' }}
          </p>
        </div>
        <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
          <p class="text-xs text-stone-500">
            {{ ui.resource.plannedRam }}
          </p>
          <p class="mt-1 text-lg font-semibold text-white">
            {{ resourcePlan?.estimatedParallelMemoryMb ? `${resourcePlan.estimatedParallelMemoryMb} MB` : 'n/a' }}
          </p>
        </div>
        <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
          <p class="text-xs text-stone-500">
            {{ etaCaption }}
          </p>
          <p class="mt-1 text-lg font-semibold text-white">
            {{ estimatedMinutesLabel }}
          </p>
        </div>
      </div>

      <div class="flex flex-wrap items-center gap-3">
        <UButton
          v-if="processing"
          color="warning"
          variant="soft"
          icon="i-lucide-square"
          :loading="cancelPending"
          @click="emit('cancelBatch')"
        >
          {{ cancelPending ? ui.resource.stopping : ui.resource.cancelBatch }}
        </UButton>
        <UBadge
          :color="resourcePlan?.canRunInParallel === false ? 'warning' : 'success'"
          variant="soft"
          :label="resourcePlan?.canRunInParallel === false ? ui.resource.parallelLimited : ui.resource.parallelOk"
        />
        <p class="text-xs leading-6 text-stone-400">
          {{ ui.resource.jobs }} {{ resourcePlan?.jobs.length ?? runQueueCount }} • {{ ui.resource.requested }} {{ effectiveParallelJobs }} • {{ ui.resource.safe }} {{ resourcePlan?.safeParallelJobs ?? 1 }}
        </p>
        <p class="text-xs leading-6 text-stone-400">
          {{ ui.resource.availableRam }} {{ liveSystemMetrics ? `${liveSystemMetrics.availableMemoryMb} MB` : 'n/a' }}
        </p>
      </div>

      <UButton
        v-if="resourcePlan?.shouldUseSequential && effectiveParallelJobs > 1"
        block
        color="warning"
        variant="soft"
        icon="i-lucide-git-commit-horizontal"
        @click="emit('enableSequentialMode')"
      >
        {{ ui.resource.sequential }}
      </UButton>

      <p
        v-if="resourcePlan?.shouldUseSequential && effectiveParallelJobs > 1"
        class="text-sm leading-6 text-amber-300"
      >
        {{ ui.resource.blocked }}
      </p>
    </div>
  </UCard>
</template>
