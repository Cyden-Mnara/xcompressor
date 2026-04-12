<script setup lang="ts">
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
</script>

<template>
  <UCard :ui="{ root: 'border border-white/10 bg-stone-950/85 ring-0' }">
    <template #header>
      <div>
        <p class="text-xs font-semibold uppercase tracking-[0.25em] text-sky-300">
          Resource planner
        </p>
        <p class="mt-2 text-sm leading-6 text-stone-300">
          {{ resourcePlanLoading ? 'Checking available resources...' : (resourcePlan?.summary || 'No jobs selected yet.') }}
        </p>
      </div>
    </template>

    <div class="space-y-3">
      <div class="grid grid-cols-2 gap-3">
        <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
          <p class="text-xs text-stone-500">
            Live CPU
          </p>
          <p class="mt-1 text-lg font-semibold text-white">
            {{ liveSystemMetrics ? `${Math.round(liveSystemMetrics.cpuUsagePercent)}%` : 'n/a' }}
          </p>
        </div>
        <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
          <p class="text-xs text-stone-500">
            Live RAM
          </p>
          <p class="mt-1 text-lg font-semibold text-white">
            {{ liveSystemMetrics ? `${liveSystemMetrics.usedMemoryMb} MB` : 'n/a' }}
          </p>
        </div>
        <div class="rounded-2xl border border-white/8 bg-black/20 p-3">
          <p class="text-xs text-stone-500">
            Planned RAM
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
          {{ cancelPending ? 'Stopping...' : 'Cancel batch' }}
        </UButton>
        <UBadge
          :color="resourcePlan?.canRunInParallel === false ? 'warning' : 'success'"
          variant="soft"
          :label="resourcePlan?.canRunInParallel === false ? 'parallel limited' : 'parallel OK'"
        />
        <p class="text-xs leading-6 text-stone-400">
          Jobs {{ resourcePlan?.jobs.length ?? runQueueCount }} • requested {{ effectiveParallelJobs }} • safe {{ resourcePlan?.safeParallelJobs ?? 1 }}
        </p>
        <p class="text-xs leading-6 text-stone-400">
          Available RAM {{ liveSystemMetrics ? `${liveSystemMetrics.availableMemoryMb} MB` : 'n/a' }}
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
        Switch to sequential mode
      </UButton>

      <p
        v-if="resourcePlan?.shouldUseSequential && effectiveParallelJobs > 1"
        class="text-sm leading-6 text-amber-300"
      >
        Parallel execution is blocked for the current estimate.
      </p>
    </div>
  </UCard>
</template>
