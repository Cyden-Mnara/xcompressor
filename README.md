# xcompressor

Tauri + Rust desktop application scaffold for batch multimedia compression.

## Current setup

- Tauri 2 desktop shell
- Nuxt 4 + Nuxt UI frontend at the repository root
- Rust backend commands for compression planning and FFmpeg batch execution
- Batch workflow for videos, images, audio, and video-to-GIF jobs

## Documentation

- [Architecture documentation](docs/architecture.md)
- [System documentation](docs/system.md)
- [Release guide](docs/releasing.md)

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

- pushes to `main`
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
- process several files in parallel through FFmpeg
- return per-file success and error details to the UI

## Next build slices

1. Stream live FFmpeg progress events into the queue.
2. Add cancellation, retries, and saved presets.
3. Expand image handling beyond FFmpeg for AVIF, JPEG, PNG, and WebP tuning.
