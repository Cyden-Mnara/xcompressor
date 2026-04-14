<script setup lang="ts">
import type { AppUiInjection } from '~/utils/app-ui'

type BatchJobResult = {
  jobId: string
  label: string | null
  inputPath: string
  mediaKind: string
  outputPath: string | null
  success: boolean
  skipped: boolean
  cancelled: boolean
  ffmpegArgs: string[]
  message: string
}

defineProps<{
  results: BatchJobResult[]
}>()
const appUi = inject('appUi') as AppUiInjection
const ui = computed(() => appUi.value)

function basename(path: string) {
  return path.split(/[\\/]/).pop() || path
}
</script>

<template>
  <UCard :ui="{ root: 'thin-scrollbar h-full overflow-y-auto border border-white/10 bg-stone-950/85 ring-0 lg:max-h-[calc(100dvh-5rem)]' }">
    <template #header>
      <div class="flex items-center justify-between gap-4">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-stone-400">
            {{ ui.output.title }}
          </p>
          <h2 class="mt-2 text-2xl font-semibold text-white">
            {{ ui.output.subtitle }}
          </h2>
        </div>
        <UBadge
          v-if="results.length"
          color="primary"
          variant="soft"
          :label="`${results.filter(result => result.success).length} ${ui.output.successful}`"
        />
      </div>
    </template>

    <div
      v-if="results.length"
      class="thin-scrollbar max-h-[60dvh] space-y-3 overflow-y-auto pr-1 lg:max-h-[calc(100dvh-17rem)]"
    >
      <div
        v-for="result in results"
        :key="result.jobId"
        class="rounded-2xl border p-3"
        :class="result.success ? 'border-emerald-500/30 bg-emerald-500/8' : (result.cancelled || result.skipped ? 'border-amber-500/30 bg-amber-500/8' : 'border-rose-500/30 bg-rose-500/8')"
      >
        <div class="flex flex-wrap items-center gap-2">
          <UBadge
            :color="result.success ? 'success' : (result.cancelled || result.skipped ? 'warning' : 'error')"
            variant="soft"
            :label="result.success ? ui.status.success : (result.cancelled ? ui.status.cancelled : (result.skipped ? ui.status.skipped : ui.status.failed))"
          />
          <UBadge
            color="neutral"
            variant="soft"
            :label="ui.media[result.mediaKind] ?? result.mediaKind"
          />
          <p class="text-sm font-medium text-white">
            {{ result.label || basename(result.inputPath) }}
          </p>
        </div>

        <p class="mt-2 line-clamp-3 text-sm leading-6 text-stone-300">
          {{ result.message }}
        </p>

        <p
          v-if="result.outputPath"
          class="mt-2 truncate text-xs text-stone-400"
        >
          {{ ui.output.output }}: {{ basename(result.outputPath) }}
        </p>

        <details
          v-if="result.ffmpegArgs.length"
          class="mt-2"
        >
          <summary class="cursor-pointer text-xs font-medium uppercase tracking-[0.2em] text-stone-400">
            {{ ui.output.args }}
          </summary>
          <pre class="mt-3 overflow-x-auto rounded-xl bg-black/40 p-3 text-xs text-stone-300">{{ result.ffmpegArgs.join(' ') }}</pre>
        </details>
      </div>
    </div>

    <UEmptyState
      v-else
      icon="i-lucide-badge-check"
      :title="ui.output.emptyTitle"
      :description="ui.output.emptyDescription"
    />
  </UCard>
</template>
