<script setup lang="ts">
const seoTitle = 'xcompressor | Open Source Media Compression'
const seoDescription = 'xcompressor is an open source desktop app for local video compression, audio conversion, image optimization, and GIF creation.'
const seoKeywords = [
  'xcompressor',
  'open source video compressor',
  'desktop media converter',
  'FFmpeg desktop app',
  'Tauri media app',
  'local video compression',
  'audio conversion',
  'GIF creator',
  'image optimization'
]

useSeoMeta({
  title: seoTitle,
  description: seoDescription,
  keywords: seoKeywords.join(', '),
  ogTitle: seoTitle,
  ogDescription: seoDescription,
  twitterCard: 'summary_large_image'
})

onMounted(() => {
  if (isTauriRuntime()) {
    void navigateTo('/app', { replace: true })
  }
})

function isTauriRuntime() {
  return import.meta.client && '__TAURI_INTERNALS__' in window
}
</script>

<template>
  <main class="min-h-screen text-stone-100">
    <div class="mx-auto w-full max-w-6xl px-4 py-6 lg:px-6">
      <nav class="mb-12 flex flex-wrap items-center justify-between gap-3">
        <NuxtLink
          to="/"
          class="flex items-center gap-3"
          aria-label="xcompressor homepage"
        >
          <img
            src="/favicon.ico"
            alt="xcompressor app icon"
            class="size-10 rounded-lg"
          >
          <span class="text-lg font-semibold text-white">xcompressor</span>
        </NuxtLink>
        <div class="flex flex-wrap gap-2">
          <UButton
            to="https://github.com/Cyden-Mnara/xcompressor/releases"
            target="_blank"
            rel="noopener noreferrer"
            icon="i-lucide-download"
            color="neutral"
            variant="soft"
          >
            Download
          </UButton>
          <UButton
            to="/credits"
            icon="i-lucide-users"
            color="neutral"
            variant="soft"
          >
            Credits
          </UButton>
          <UButton
            to="/support"
            icon="i-lucide-heart-handshake"
            color="primary"
            variant="soft"
          >
            Support
          </UButton>
        </div>
      </nav>

      <section class="grid min-h-[calc(100dvh-11rem)] items-center gap-10 lg:grid-cols-[minmax(0,1fr)_360px]">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-amber-300">
            Open source desktop media tool
          </p>
          <h1 class="mt-5 max-w-4xl text-5xl font-semibold tracking-tight text-white sm:text-6xl">
            Compress, convert, and shape media without sending it away.
          </h1>
          <p class="mt-6 max-w-2xl text-lg leading-8 text-stone-300">
            xcompressor is a local-first desktop workspace for video, image, audio, and GIF jobs. Add files, tune the output, run a batch, and keep your media on your machine.
          </p>
          <div class="mt-8 flex flex-wrap gap-3">
            <UButton
              to="https://github.com/Cyden-Mnara/xcompressor/releases"
              target="_blank"
              rel="noopener noreferrer"
              icon="i-lucide-download"
              color="primary"
              size="lg"
            >
              Download releases
            </UButton>
            <UButton
              to="https://github.com/Cyden-Mnara/xcompressor"
              target="_blank"
              rel="noopener noreferrer"
              icon="i-lucide-git-commit-horizontal"
              color="neutral"
              variant="soft"
              size="lg"
            >
              View source
            </UButton>
          </div>
        </div>

        <div class="mx-auto grid w-full max-w-sm place-items-center">
          <img
            src="/favicon.ico"
            alt="xcompressor desktop app icon"
            class="size-48 rounded-lg border border-white/10 bg-stone-950/85 p-8 shadow-2xl sm:size-56"
          >
        </div>
      </section>

      <section class="grid gap-4 py-8 md:grid-cols-3">
        <article class="rounded-lg border border-white/10 bg-stone-950/85 p-5">
          <UIcon
            name="i-lucide-video"
            class="size-7 text-amber-300"
          />
          <h2 class="mt-4 text-xl font-semibold text-white">
            Batch Media Work
          </h2>
          <p class="mt-3 text-sm leading-6 text-stone-300">
            Queue videos, images, and audio files, then run the work without rebuilding the same settings for every file.
          </p>
        </article>

        <article class="rounded-lg border border-white/10 bg-stone-950/85 p-5">
          <UIcon
            name="i-lucide-film"
            class="size-7 text-amber-300"
          />
          <h2 class="mt-4 text-xl font-semibold text-white">
            FFmpeg Powered
          </h2>
          <p class="mt-3 text-sm leading-6 text-stone-300">
            Built around FFmpeg workflows for dependable compression, conversion, previewing, and GIF generation.
          </p>
        </article>

        <article class="rounded-lg border border-white/10 bg-stone-950/85 p-5">
          <UIcon
            name="i-lucide-monitor-play"
            class="size-7 text-amber-300"
          />
          <h2 class="mt-4 text-xl font-semibold text-white">
            Local First
          </h2>
          <p class="mt-3 text-sm leading-6 text-stone-300">
            Process files on your own desktop. No account, upload queue, or cloud round trip is needed for the core workflow.
          </p>
        </article>
      </section>

      <section class="grid gap-5 py-8 lg:grid-cols-[0.8fr_1.2fr]">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.25em] text-amber-300">
            Why it exists
          </p>
          <h2 class="mt-4 text-3xl font-semibold text-white">
            A practical tool for everyday media cleanup.
          </h2>
        </div>
        <div class="space-y-4 text-sm leading-7 text-stone-300">
          <p>
            xcompressor is made for people who regularly need smaller videos, converted audio, optimized images, or quick GIF clips, but do not want a complicated editor for simple output work.
          </p>
          <p>
            The project is open source so users can inspect how media is handled, contributors can improve the workflow, and upstream projects receive visible credit.
          </p>
        </div>
      </section>
    </div>
  </main>
</template>
