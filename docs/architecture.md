# Architecture Documentation

## Overview

`xcompressor` is a desktop multimedia processing application built with:

- `Tauri 2` for the desktop shell
- `Nuxt 4 + Nuxt UI` for the frontend
- `Rust` for backend orchestration
- `FFmpeg / FFprobe` for media processing and metadata probing

The system is split into three main layers:

1. Presentation layer
2. Application orchestration layer
3. Media execution layer

## High-Level Architecture

```mermaid
flowchart TD
  UI[Nuxt UI Frontend<br/>app/pages/index.vue<br/>workspace layout<br/>queue editing<br/>GIF clip editor<br/>progress and results]
  TAURI[Rust Tauri Backend<br/>src-tauri/src/lib.rs<br/>command handlers<br/>resource planning<br/>job queueing<br/>parallel worker control<br/>FFmpeg command building]
  MEDIA[Native Media Tooling<br/>ffmpeg / ffprobe<br/>encode / convert<br/>GIF generation<br/>duration probing]

  UI -->|invoke / event bridge| TAURI
  TAURI -->|spawn processes| MEDIA
```

## Frontend Architecture

The frontend is currently centered in [app/pages/index.vue](../app/pages/index.vue).

### Responsibilities

- render the desktop workspace
- maintain local UI state for:
  - selected files
  - output directory
  - active mode
  - GIF clip ranges
  - mixed activity queue
  - progress and results
- call backend commands with `invoke`
- subscribe to `batch-progress` events
- enforce UX-level rules such as:
  - GIF mode requiring queued clips
  - sequential fallback when resource planning says parallel execution is unsafe

### Main frontend state groups

- source selection state
- operation configuration state
- GIF editor state
- saved mixed-activity state
- batch progress state
- resource planning state

### Frontend interaction model

The page uses a workspace layout with three logical columns:

- left: overview and capability context
- center: configuration, GIF editor, queue management
- right: resource planning, monitor, and batch output

## Backend Architecture

The Rust backend is implemented in [src-tauri/src/lib.rs](../src-tauri/src/lib.rs).

### Tauri commands

The application exposes these command entry points:

- `get_app_bootstrap`
- `plan_compression`
- `analyze_resource_plan`
- `check_for_app_update`
- `install_app_update`
- `cancel_batch_run`
- `open_media_in_system_player`
- `run_batch_jobs`

### Backend responsibilities

- expose frontend bootstrap metadata
- check and install signed application updates from GitHub Releases
- analyze planned workload against available system resources
- build FFmpeg argument lists by media kind and operation
- resolve bundled FFmpeg / FFprobe binaries inside packaged builds
- manage parallel execution with worker threads
- emit batch progress events back to the frontend
- return structured per-job results

## Job Model

There are three job modes:

- `compress`
- `convert`
- `gif`

There are two queueing patterns:

1. Simple batch
   - one mode applied to many inputs
2. Mixed activity batch
   - each queued item carries its own mode and settings

### Core request types

- `BatchProcessRequest`
- `MixedJobRequest`
- `GifSegmentRequest`
- `ResourcePlanRequest`

### Core response/event types

- `BatchProcessResponse`
- `BatchJobResult`
- `BatchProgressEvent`
- `ResourcePlan`

## Media Execution Layer

The backend does not implement codecs itself. It delegates media work to FFmpeg.

### Why FFmpeg is the execution boundary

- codec support is already mature
- conversion and compression are cross-platform
- GIF generation is reliable through filter graphs
- probing duration with `ffprobe` is simpler than custom parsing

### FFmpeg-related backend responsibilities

- choose output format per media kind
- choose codec parameters from preset profile
- apply resize filters when configured
- generate GIF palette/filter arguments
- estimate progress from `out_time_us` and probed duration

## Parallelism Model

Parallelism is controlled in Rust through a worker queue:

- queued jobs are stored in a shared `VecDeque`
- worker threads pop from the queue
- each worker processes one job at a time
- results are collected and sorted back into request order

Parallel execution is constrained by:

- user-selected `maxParallelJobs`
- resource planner recommendation

## Resource Planning Model

Resource planning is advisory plus protective.

The planner:

- reads logical CPU core count
- reads memory availability from `/proc/meminfo` on supported environments
- estimates RAM and duration per job using media kind, operation type, and file size
- derives a safe parallelism count

The frontend uses this to:

- surface ETA and memory pressure
- disable unsafe parallel runs
- suggest sequential mode

## Platform and Build Architecture

### Local development

- frontend dev server: Nuxt
- desktop runtime: Tauri
- backend build: Cargo

### CI

The GitHub workflow in [.github/workflows/build-desktop.yml](../.github/workflows/build-desktop.yml) builds:

- Windows artifacts
- macOS artifacts

Tagged releases are published through [.github/workflows/publish-release.yml](../.github/workflows/publish-release.yml), which uploads signed updater artifacts to GitHub Releases.

## Current Architectural Constraints

- most frontend logic is still in a single page component
- preview playback depends on webview codec support
- resource planning uses heuristics, not real-time process telemetry
- updater availability depends on release signing keys and GitHub Releases metadata being configured in CI

## Recommended Next Refactors

- split the frontend into feature components:
  - workspace shell
  - GIF editor
  - mixed activity queue
  - results monitor
- extract Rust job planning/execution into internal modules
- add retry primitives and persistent job history
- add persistent presets and saved jobs
