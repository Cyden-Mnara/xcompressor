//! Tauri backend for xcompressor.
//!
//! This crate exposes desktop commands for media previewing, compression
//! planning, batch execution, resource estimation, cancellation, and updater
//! integration. Long-running batch cancellation state lives in the internal
//! batch control module so the command layer can stay focused on request handling.

mod batch;
mod batch_control;
mod bootstrap;
mod metrics;
mod preview;
mod updater;

use batch_control::{cancel_active_batch_runs, BatchRunControl};
use preview::MediaPreviewServer;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use sysinfo::System;
use tauri::{Manager, WindowEvent};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompressionPreset {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    quality_range: &'static str,
    size_reduction_range: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaCapability {
    kind: &'static str,
    compression_modes: Vec<&'static str>,
    conversions: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FormatTargets {
    kind: &'static str,
    targets: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppBootstrap {
    app_name: &'static str,
    version: &'static str,
    summary: &'static str,
    presets: Vec<CompressionPreset>,
    media_capabilities: Vec<MediaCapability>,
    format_targets: Vec<FormatTargets>,
    gif_workflow: Vec<&'static str>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompressionPlanRequest {
    media_kind: String,
    goal: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompressionPlan {
    media_kind: String,
    goal: String,
    recommended_preset: &'static str,
    estimated_output_range: &'static str,
    notes: Vec<&'static str>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GifOptions {
    start_seconds: f64,
    duration_seconds: f64,
    fps: u16,
    width: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GifSegmentRequest {
    job_id: String,
    input_path: String,
    output_suffix: String,
    label: Option<String>,
    start_seconds: f64,
    duration_seconds: f64,
    fps: u16,
    width: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchProcessRequest {
    run_id: String,
    input_paths: Vec<String>,
    output_dir: String,
    mode: String,
    preset_id: String,
    video_format: String,
    image_format: String,
    audio_format: String,
    resize_long_edge: Option<u32>,
    max_parallel_jobs: Option<usize>,
    gif: Option<GifOptions>,
    gif_segments: Option<Vec<GifSegmentRequest>>,
    mixed_jobs: Option<Vec<MixedJobRequest>>,
    overwrite: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MixedJobRequest {
    job_id: String,
    label: Option<String>,
    input_path: String,
    output_dir: String,
    mode: String,
    preset_id: String,
    video_format: String,
    image_format: String,
    audio_format: String,
    resize_long_edge: Option<u32>,
    gif: Option<GifOptions>,
    output_suffix: Option<String>,
    overwrite: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchJobResult {
    job_id: String,
    label: Option<String>,
    input_path: String,
    media_kind: String,
    operation: String,
    output_path: Option<String>,
    success: bool,
    skipped: bool,
    cancelled: bool,
    ffmpeg_args: Vec<String>,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchProcessResponse {
    results: Vec<BatchJobResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchProgressEvent {
    job_id: String,
    label: Option<String>,
    input_path: String,
    media_kind: String,
    operation: String,
    status: String,
    progress_percent: Option<f64>,
    output_path: Option<String>,
    message: String,
    speed: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResourcePlanRequest {
    input_paths: Vec<String>,
    mode: String,
    max_parallel_jobs: Option<usize>,
    gif_segments: Option<Vec<GifSegmentRequest>>,
    mixed_jobs: Option<Vec<MixedJobRequest>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceJobEstimate {
    job_id: String,
    label: String,
    media_kind: String,
    operation: String,
    estimated_memory_mb: u64,
    estimated_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourcePlan {
    logical_cores: usize,
    available_memory_mb: Option<u64>,
    total_memory_mb: Option<u64>,
    max_parallel_jobs: usize,
    safe_parallel_jobs: usize,
    estimated_parallel_memory_mb: u64,
    estimated_total_seconds: u64,
    estimated_parallel_seconds: u64,
    can_run_in_parallel: bool,
    should_use_sequential: bool,
    summary: String,
    jobs: Vec<ResourceJobEstimate>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LiveSystemMetrics {
    cpu_usage_percent: f32,
    used_memory_mb: u64,
    available_memory_mb: u64,
    total_memory_mb: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppUpdateStatus {
    configured: bool,
    current_version: String,
    available_version: Option<String>,
    notes: Option<String>,
    pub_date: Option<String>,
    update_ready: bool,
    update_installed: bool,
    message: String,
}

struct AppState {
    batch_runs: Arc<Mutex<HashMap<String, Arc<BatchRunControl>>>>,
    preview_server: MediaPreviewServer,
    system: Arc<Mutex<System>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let preview_server =
        preview::start_media_preview_server().expect("failed to start local media preview server");

    tauri::Builder::default()
        .manage(AppState {
            batch_runs: Arc::new(Mutex::new(HashMap::new())),
            preview_server,
            system: Arc::new(Mutex::new(System::new_all())),
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            bootstrap::get_app_bootstrap,
            metrics::get_live_system_metrics,
            bootstrap::plan_compression,
            batch::analyze_resource_plan,
            preview::open_media_in_system_player,
            preview::get_media_preview_url,
            updater::check_for_app_update,
            updater::install_app_update,
            batch::cancel_batch_run,
            batch::run_batch_jobs
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, WindowEvent::CloseRequested { .. }) {
                let state = window.state::<AppState>();
                cancel_active_batch_runs(&state.batch_runs);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
