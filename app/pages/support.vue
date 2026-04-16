<script setup lang="ts">
const coffeeUrl = 'https://www.buymeacoffee.com/xcompressor'
const seoTitle = 'Support xcompressor | Keep the Project Alive'
const seoDescription = 'Support xcompressor development, maintenance, testing, documentation, and open source multimedia tooling.'
const seoKeywords = [
  'support xcompressor',
  'buy me a coffee xcompressor',
  'sponsor open source',
  'support FFmpeg app',
  'open source video compressor',
  'multimedia compression tool',
  'desktop media converter'
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
          to="/credits"
          icon="i-lucide-users"
          color="neutral"
          variant="soft"
        >
          Credits
        </UButton>
      </nav>

      <section class="rounded-lg border border-white/10 bg-stone-950/85 p-6 shadow-2xl">
        <p class="text-xs font-semibold uppercase tracking-[0.25em] text-amber-300">
          Support
        </p>
        <h1 class="mt-4 text-4xl font-semibold tracking-tight text-white sm:text-5xl">
          Help keep xcompressor alive.
        </h1>
        <p class="mt-5 max-w-3xl text-base leading-7 text-stone-300">
          Optional contributions help cover xcompressor testing time, release work, documentation, bug fixes, packaging, and maintenance.
        </p>
        <div class="mt-6 flex flex-wrap gap-3">
          <UButton
            :to="coffeeUrl"
            target="_blank"
            rel="noopener noreferrer"
            icon="i-lucide-coffee"
            color="primary"
            size="lg"
          >
            Buy me a coffee
          </UButton>
          <UButton
            to="/credits"
            icon="i-lucide-heart-handshake"
            color="neutral"
            variant="soft"
            size="lg"
          >
            See acknowledgements
          </UButton>
        </div>
      </section>

      <div class="mt-5 grid gap-5 lg:grid-cols-3">
        <section class="rounded-lg border border-white/10 bg-stone-950/85 p-5">
          <h2 class="text-xl font-semibold text-white">
            Maintenance
          </h2>
          <p class="mt-3 text-sm leading-6 text-stone-300">
            Keep FFmpeg workflows current, fix regressions, review issues, and package releases for desktop users.
          </p>
        </section>
        <section class="rounded-lg border border-white/10 bg-stone-950/85 p-5">
          <h2 class="text-xl font-semibold text-white">
            Quality
          </h2>
          <p class="mt-3 text-sm leading-6 text-stone-300">
            Test with real media files, improve failure messages, and make compression results easier to understand.
          </p>
        </section>
        <section class="rounded-lg border border-white/10 bg-stone-950/85 p-5">
          <h2 class="text-xl font-semibold text-white">
            Community
          </h2>
          <p class="mt-3 text-sm leading-6 text-stone-300">
            Write docs, welcome contributors, credit upstream projects, and keep the project respectful and practical.
          </p>
        </section>
      </div>

      <section class="mt-5 rounded-lg border border-white/10 bg-stone-950/85 p-5">
        <h2 class="text-2xl font-semibold text-white">
          Donation Policy
        </h2>
        <p class="mt-4 text-sm leading-6 text-stone-300">
          Support is not payment for FFmpeg, codecs, patent licenses, warranty, private support, priority fixes, or ownership rights. FFmpeg remains an independent open source project with its own licenses and maintainers.
        </p>
      </section>

      <section class="mt-5 rounded-lg border border-white/10 bg-stone-950/85 p-5">
        <h2 class="text-2xl font-semibold text-white">
          Other Ways To Help
        </h2>
        <div class="mt-4 grid gap-3 text-sm text-stone-300 sm:grid-cols-2">
          <div class="rounded-lg bg-white/5 p-3">
            Report bugs with sample settings and platform details.
          </div>
          <div class="rounded-lg bg-white/5 p-3">
            Improve documentation for new users.
          </div>
          <div class="rounded-lg bg-white/5 p-3">
            Test release candidates on Windows, macOS, and Linux.
          </div>
          <div class="rounded-lg bg-white/5 p-3">
            Share careful feedback about quality, speed, and file size.
          </div>
        </div>
      </section>
    </div>
  </main>
</template>
