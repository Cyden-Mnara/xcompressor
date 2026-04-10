# xcompressor

Tauri + Rust desktop application scaffold for batch multimedia compression.

## Current setup

- Tauri 2 desktop shell
- Nuxt 4 + Nuxt UI frontend at the repository root
- Rust backend commands for compression planning and FFmpeg batch execution
- Batch workflow for videos, images, audio, and video-to-GIF jobs

## Run

```bash
pnpm dev
```

You can also run everything from the project root:

```bash
pnpm rustcheck
pnpm check
pnpm frontendbuild
pnpm tauribuild
```

`pnpm dev` uses `cargo tauri dev --no-watch` to avoid Linux inotify exhaustion on systems with many active file watchers. If you want the original Tauri Rust watcher, use:

```bash
pnpm tauridevwatch
```

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
