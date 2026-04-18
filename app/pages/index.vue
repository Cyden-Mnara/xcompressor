<script setup lang="ts">
import type { DeviceProfile, DownloadOs, GitHubRelease, OsDownloadCard, ReleaseDownload, ReleaseJson, ReleasePlatform } from '~/types/interfaces'
import { fileNameFromUrl } from '~/utils/app.autodownloader'

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

const releaseApiUrl = 'https://api.github.com/repos/Cyden-Mnara/xcompressor/releases/latest'
const releaseFallbackUrl = 'https://github.com/Cyden-Mnara/xcompressor/releases'

const release = ref<ReleaseJson | null>(null)
const releaseLoading = ref(false)
const releaseError = ref('')
const preparingDownload = ref<DownloadOs | ''>('')
const preparedDownloadName = ref('')

const osDownloadCards: OsDownloadCard[] = [
  {
    os: 'windows',
    label: 'Windows',
    icon: 'i-simple-icons-windows',
    description: 'Download for Windows'
  },
  {
    os: 'macos',
    label: 'macOS',
    icon: 'i-simple-icons-apple',
    description: 'Download for macOS'
  },
  {
    os: 'linux',
    label: 'Linux',
    icon: 'i-simple-icons-linux',
    description: 'Download for Linux'
  }
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

async function prepareDownload(os: DownloadOs) {
  releaseError.value = ''
  preparedDownloadName.value = ''
  preparingDownload.value = os
  releaseLoading.value = true

  try {
    const [releaseData, profile] = await Promise.all([
      loadReleaseDownloads(),
      detectDeviceProfile()
    ])
    const download = releaseData ? pickDownloadForOs(releaseData, os, profile) : null

    if (!download) {
      throw new Error(`No ${labelForOs(os)} download was found in the latest release.`)
    }

    preparedDownloadName.value = download.fileName
    startDownload(download.url)
  } catch (error) {
    releaseError.value = error instanceof Error ? error.message : 'Downloads are temporarily unavailable.'
  } finally {
    releaseLoading.value = false
    preparingDownload.value = ''
  }
}

async function loadReleaseDownloads() {
  try {
    if (release.value) {
      return release.value
    }

    const latestRelease = await $fetch<GitHubRelease>(releaseApiUrl)
    release.value = normalizeGitHubRelease(latestRelease)
    return release.value
  } catch (error) {
    console.error('Error loading release metadata:', error)
  }
}

function normalizeGitHubRelease(githubRelease: GitHubRelease): ReleaseJson {
  return {
    version: githubRelease.tag_name,
    notes: githubRelease.body,
    pub_date: githubRelease.published_at,
    platforms: Object.fromEntries((githubRelease.assets ?? [])
      .filter(asset => isDownloadAsset(asset.name))
      .map(asset => [
        asset.name,
        {
          url: asset.browser_download_url
        }
      ]))
  }
}

function isDownloadAsset(name: string) {
  const normalized = name.toLowerCase()

  return !normalized.endsWith('.sig') && normalized !== 'latest.json' && !normalized.endsWith('.json')
}

async function detectDeviceProfile(): Promise<DeviceProfile> {
  const profile: DeviceProfile = {
    architecture: '',
    bitness: '',
    platformVersion: ''
  }
  const navigatorWithUserAgentData = navigator as Navigator & {
    userAgentData?: {
      getHighEntropyValues?: (hints: string[]) => Promise<Partial<DeviceProfile>>
    }
  }

  if (navigatorWithUserAgentData.userAgentData?.getHighEntropyValues) {
    const values = await navigatorWithUserAgentData.userAgentData.getHighEntropyValues([
      'architecture',
      'bitness',
      'platformVersion'
    ])

    profile.architecture = values.architecture ?? ''
    profile.bitness = values.bitness ?? ''
    profile.platformVersion = values.platformVersion ?? ''
  }

  return profile
}

function pickDownloadForOs(releaseData: ReleaseJson, os: DownloadOs, profile: DeviceProfile): ReleaseDownload | null {
  const downloads = Object.entries(releaseData.platforms ?? {})
    .filter((entry): entry is [string, ReleasePlatform & { url: string }] => Boolean(entry[1].url))
    .map(([target, platform]) => ({
      target,
      url: platform.url,
      fileName: fileNameFromUrl(platform.url)
    }))
    .filter(download => targetMatchesOs(download.target, os))
    .sort((first, second) => downloadPriority(first, os, profile) - downloadPriority(second, os, profile))

  return downloads.find(download => targetMatchesArchitecture(download.target, profile))
    ?? downloads[0]
    ?? null
}

function downloadPriority(download: ReleaseDownload, os: DownloadOs, profile: DeviceProfile) {
  const target = download.target.toLowerCase()
  const architecturePenalty = targetMatchesArchitecture(target, profile) ? 0 : 100

  if (os === 'linux') {
    return architecturePenalty + linuxPackagePriority(target)
  }

  if (os === 'windows') {
    return architecturePenalty + (target.endsWith('.msi') ? 0 : target.endsWith('.exe') ? 1 : 2)
  }

  return architecturePenalty + (target.endsWith('.dmg') ? 0 : 1)
}

function linuxPackagePriority(target: string) {
  const environment = `${navigator.userAgent} ${navigator.platform}`.toLowerCase()
  const prefersRpm = environment.includes('fedora')
    || environment.includes('rhel')
    || environment.includes('red hat')
    || environment.includes('opensuse')
    || environment.includes('suse')

  if (prefersRpm) {
    return target.endsWith('.rpm') ? 0 : target.endsWith('.appimage') ? 1 : target.endsWith('.deb') ? 2 : 3
  }

  return target.endsWith('.deb') ? 0 : target.endsWith('.appimage') ? 1 : target.endsWith('.rpm') ? 2 : 3
}

function targetMatchesOs(target: string, os: DownloadOs) {
  const normalized = target.toLowerCase()

  if (os === 'windows') {
    return normalized.includes('windows')
      || normalized.endsWith('.msi')
      || normalized.endsWith('.exe')
  }

  if (os === 'macos') {
    return normalized.includes('darwin')
      || normalized.includes('macos')
      || normalized.includes('apple')
      || normalized.endsWith('.dmg')
  }

  return normalized.includes('linux')
    || normalized.endsWith('.appimage')
    || normalized.endsWith('.deb')
    || normalized.endsWith('.rpm')
}

function targetMatchesArchitecture(target: string, profile: DeviceProfile) {
  const normalized = target.toLowerCase()
  const architecture = `${profile.architecture} ${profile.bitness} ${navigator.userAgent} ${navigator.platform}`.toLowerCase()

  if (architecture.includes('arm') || architecture.includes('aarch64')) {
    return normalized.includes('aarch64') || normalized.includes('arm64')
  }

  if (architecture.includes('x86') || architecture.includes('64') || architecture.includes('amd64')) {
    return normalized.includes('x86_64') || normalized.includes('amd64') || normalized.includes('x64')
  }

  return true
}

function labelForOs(os: DownloadOs) {
  return osDownloadCards.find(card => card.os === os)?.label ?? os
}

function startDownload(url: string) {
  const link = document.createElement('a')
  link.href = url
  link.rel = 'noopener noreferrer'
  link.download = ''
  document.body.appendChild(link)
  link.click()
  link.remove()
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
            to="#download"
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
              to="#download"
              icon="i-lucide-download"
              color="primary"
              size="lg"
            >
              Download
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

      <section
        id="download"
        class="py-8"
      >
        <div class="grid gap-5 lg:grid-cols-[0.8fr_1.2fr]">
          <div>
            <p class="text-xs font-semibold uppercase tracking-[0.25em] text-amber-300">
              Desktop downloads
            </p>
            <h2 class="mt-4 text-3xl font-semibold text-white">
              Pick your system. The right build, download will starts automatically.
            </h2>

            <div class="mt-4 min-h-6 text-sm text-stone-300">
              <p
                v-if="preparingDownload"
                class="flex items-center gap-2"
              >
                <UIcon
                  name="i-lucide-refresh-cw"
                  class="size-4 animate-spin text-amber-300"
                />
                Preparing your download. It will start automatically.
              </p>
              <p
                v-else-if="preparedDownloadName"
                class="text-amber-300"
              >
                Starting {{ preparedDownloadName }}.
              </p>
              <p
                v-else-if="releaseError"
                class="text-red-300"
              >
                {{ releaseError }}
              </p>
            </div>

            <UButton
              v-if="releaseError"
              :to="releaseFallbackUrl"
              target="_blank"
              rel="noopener noreferrer"
              icon="i-lucide-external-link"
              color="neutral"
              variant="soft"
              class="mt-4"
            >
              Open releases
            </UButton>
          </div>

          <div class="grid gap-4 sm:grid-cols-3">
            <button
              v-for="card in osDownloadCards"
              :key="card.os"
              type="button"
              class="group flex min-h-44 flex-col rounded-lg border border-white/10 bg-stone-950/85 p-5 text-left transition hover:-translate-y-0.5 hover:border-amber-300/60 hover:bg-white/8 disabled:cursor-wait disabled:opacity-70 disabled:hover:translate-y-0"
              :disabled="releaseLoading"
              @click="prepareDownload(card.os)"
            >
              <span class="flex size-12 items-center justify-center rounded-lg border border-white/10 bg-black/20 text-amber-300">
                <UIcon
                  :name="card.icon"
                  class="size-6"
                />
              </span>
              <span class="mt-5 text-lg font-semibold text-white">
                {{ card.description }}
              </span>
              <span class="mt-2 min-h-10 text-sm leading-5 text-stone-300">
                <template v-if="preparingDownload === card.os">
                  Reading release metadata and matching your device.
                </template>
                <template v-else>
                  Latest {{ card.label }} desktop build.
                </template>
              </span>
            </button>
          </div>
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
