# xcompressor

Tauri + Rust desktop application scaffold for batch multimedia compression.

## Current setup

- Tauri 2 desktop shell
- Nuxt 4 + Nuxt UI frontend at the repository root
- Rust backend commands for compression planning and FFmpeg batch execution
- Batch workflow for videos, images, audio, and video-to-GIF jobs
- Single-worker shortest-job-first scheduling to keep the desktop responsive

## Documentation

- [Architecture documentation](docs/architecture.md)
- [System documentation](docs/system.md)
- [Release guide](docs/releasing.md)
- [Open source release checklist](docs/open-source-release-checklist.md)
- [End user notice draft](docs/end-user-notice.md)
- [Contribution guide](CONTRIBUTING.md)
- [Code of conduct](CODE_OF_CONDUCT.md)

## Credits and support

xcompressor is built on FFmpeg and the wider open source multimedia ecosystem. The app includes static pages for:

- `/credits` - acknowledgements for FFmpeg developers and open source contributors
- `/support` - non-financial ways to support the project

Financial support is not currently enabled. Keep donation and sponsorship links unpublished until there is a working payout method.

If release builds bundle FFmpeg binaries, also publish the matching FFmpeg source, license notices, and build details required for the exact binaries you distribute. Start with the official FFmpeg legal guidance: https://ffmpeg.org/legal.html

Future donations, if enabled, should be optional support for xcompressor maintenance only. They must not be presented as payment for FFmpeg, codec licenses, patent rights, warranty, private support, or ownership rights.

## Run

```bash
pnpm dev
```

You can also run everything from the project root:

```bash
pnpm rustcheck
pnpm check
pnpm generate
pnpm tauribuild
```

`pnpm tauri` uses `cargo tauri dev --no-watch` to avoid Linux inotify exhaustion on systems with many active file watchers. If you want the original Tauri Rust watcher, use:

```bash
pnpm tauriwatch
```

## GitHub Actions desktop builds

This repo includes [.github/workflows/build-desktop.yml](.github/workflows/build-desktop.yml), which builds desktop bundles for:

- Windows
- macOS

The workflow runs on:

- pushes to `master`
- pull requests
- manual workflow dispatch

Build outputs are uploaded as GitHub Actions artifacts for each workflow run.

Tagged releases use [.github/workflows/publish-release.yml](.github/workflows/publish-release.yml) to publish signed bundles and Tauri updater metadata to GitHub Releases.

## Bundled FFmpeg

Packaged CI builds stage `ffmpeg` and `ffprobe` into Tauri resources so released apps can run without depending on a system-wide FFmpeg install. Local development still falls back to `ffmpeg` and `ffprobe` on `PATH`.

## Implemented batch pipeline

- pick multiple source files
- choose one output directory
- run `compress`, `convert`, or `gif` jobs
- process one FFmpeg job at a time
- order submitted jobs by shortest estimated runtime first
- keep jobs added during a run waiting for the next batch
- cancel tracked FFmpeg processes when a run is cancelled or the app closes
- return per-file success and error details to the UI

## Next build slices

1. Add retries and failure recovery for cancelled or failed outputs.
2. Add persistent settings and saved presets.
3. Expand image handling beyond FFmpeg for AVIF, JPEG, PNG, and WebP tuning.
