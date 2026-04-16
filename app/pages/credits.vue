<script setup lang="ts">
const seoTitle = 'Credits and Open Source Acknowledgements | xcompressor'
const seoDescription = 'xcompressor credits FFmpeg developers, open source maintainers, translators, testers, and the wider community behind multimedia compression tools.'
const seoKeywords = [
  'xcompressor credits',
  'FFmpeg developers',
  'open source multimedia',
  'video compression app',
  'audio conversion tool',
  'GIF generator',
  'Tauri FFmpeg app',
  'Nuxt desktop app',
  'media converter open source'
]

useSeoMeta({
  title: seoTitle,
  description: seoDescription,
  keywords: seoKeywords.join(', '),
  ogTitle: seoTitle,
  ogDescription: seoDescription,
  twitterCard: 'summary_large_image'
})
</script>

<template>
  <main class="min-h-screen text-stone-100">
    <div class="mx-auto flex min-h-screen w-full max-w-5xl flex-col px-4 py-6 lg:px-6">
      <nav class="mb-8 flex flex-wrap items-center justify-between gap-3">
        <UButton
          to="/"
          icon="i-lucide-arrow-left"
          color="neutral"
          variant="soft"
        >
          Back to app
        </UButton>
        <UButton
          to="/support"
          icon="i-lucide-coffee"
          color="primary"
          variant="soft"
        >
          Support xcompressor
        </UButton>
      </nav>

      <section class="rounded-lg border border-white/10 bg-stone-950/85 p-6 shadow-2xl">
        <p class="text-xs font-semibold uppercase tracking-[0.25em] text-amber-300">
          Credits
        </p>
        <h1 class="mt-4 text-4xl font-semibold tracking-tight text-white sm:text-5xl">
          Built on generous open source work.
        </h1>
        <p class="mt-5 max-w-3xl text-base leading-7 text-stone-300">
          xcompressor exists because thousands of developers, maintainers, documenters, translators, and testers make multimedia software better in public.
        </p>
      </section>

      <div class="mt-5 grid gap-5 lg:grid-cols-2">
        <section class="rounded-lg border border-white/10 bg-stone-950/85 p-5">
          <div class="flex items-center gap-3">
            <UIcon
              name="i-lucide-film"
              class="size-6 text-amber-300"
            />
            <h2 class="text-2xl font-semibold text-white">
              FFmpeg
            </h2>
          </div>
          <p class="mt-4 text-sm leading-6 text-stone-300">
            FFmpeg is the engine that makes reliable video, audio, image, and GIF processing possible across formats and platforms. xcompressor thanks the FFmpeg developers and contributors for their long-running work on codecs, filters, command-line tooling, documentation, and cross-platform builds.
          </p>
          <div class="mt-5 flex flex-wrap gap-2">
            <UButton
              to="https://ffmpeg.org/"
              target="_blank"
              rel="noopener noreferrer"
              icon="i-lucide-external-link"
              color="neutral"
              variant="soft"
              size="sm"
            >
              FFmpeg project
            </UButton>
            <UButton
              to="https://ffmpeg.org/legal.html"
              target="_blank"
              rel="noopener noreferrer"
              icon="i-lucide-scroll-text"
              color="neutral"
              variant="ghost"
              size="sm"
            >
              FFmpeg legal notes
            </UButton>
          </div>
        </section>

        <section class="rounded-lg border border-white/10 bg-stone-950/85 p-5">
          <div class="flex items-center gap-3">
            <UIcon
              name="i-lucide-users"
              class="size-6 text-amber-300"
            />
            <h2 class="text-2xl font-semibold text-white">
              Open Source Community
            </h2>
          </div>
          <p class="mt-4 text-sm leading-6 text-stone-300">
            This project also depends on Rust, Tauri, Nuxt, Vue, Nuxt UI, Tailwind CSS, Iconify, and many smaller libraries that quietly carry important work. Every issue report, package release, security fix, accessibility improvement, and translation helps keep tools like this alive.
          </p>
          <div class="mt-5 grid gap-3 text-sm text-stone-300 sm:grid-cols-2">
            <div class="rounded-lg bg-white/5 p-3">
              Compression and conversion workflows
            </div>
            <div class="rounded-lg bg-white/5 p-3">
              Cross-platform desktop packaging
            </div>
            <div class="rounded-lg bg-white/5 p-3">
              Interface libraries and icons
            </div>
            <div class="rounded-lg bg-white/5 p-3">
              Tests, reports, and documentation
            </div>
          </div>
        </section>
      </div>

      <section class="mt-5 rounded-lg border border-white/10 bg-stone-950/85 p-5">
        <h2 class="text-2xl font-semibold text-white">
          Project Promise
        </h2>
        <p class="mt-4 text-sm leading-6 text-stone-300">
          xcompressor should remain useful, transparent, respectful of upstream projects, and honest about what it builds on. When this project uses open source work, it should preserve licenses, keep attribution visible, and send fixes upstream whenever practical.
        </p>
      </section>

      <section class="mt-5 rounded-lg border border-white/10 bg-stone-950/85 p-5">
        <h2 class="text-2xl font-semibold text-white">
          FFmpeg Licensing Note
        </h2>
        <p class="mt-4 text-sm leading-6 text-stone-300">
          Released builds that include FFmpeg must ship with the right license notices, matching source availability, and build details for the exact FFmpeg binaries distributed with the app. Review the official FFmpeg legal guidance before publishing downloads.
        </p>
        <UButton
          to="https://ffmpeg.org/legal.html"
          target="_blank"
          rel="noopener noreferrer"
          icon="i-lucide-external-link"
          color="neutral"
          variant="soft"
          size="sm"
          class="mt-5"
        >
          Review FFmpeg legal guidance
        </UButton>
      </section>
    </div>
  </main>
</template>
