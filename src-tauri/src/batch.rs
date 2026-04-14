//! Batch resource planning and FFmpeg execution.
//!
//! This module keeps FFmpeg argument construction, progress parsing, resource
//! estimates, and batch worker scheduling out of the Tauri app bootstrap.

use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

use tauri::{AppHandle, Emitter, Manager, State};

use crate::{
    batch_control::{terminate_process, BatchRunControl},
    AppState, BatchJobResult, BatchProcessRequest, BatchProcessResponse, BatchProgressEvent,
    GifOptions, GifSegmentRequest, MixedJobRequest, ResourceJobEstimate, ResourcePlan,
    ResourcePlanRequest,
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

enum RunFfmpegError {
    Failed(String),
    Cancelled,
}

#[derive(Debug)]
struct PresetProfile {
    video_crf: u8,
    video_webm_crf: u8,
    audio_bitrate_kbps: u16,
    image_quality: u8,
}

fn detect_media_kind(path: &Path) -> &'static str {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_lowercase();

    match extension.as_str() {
        "mp4" | "mov" | "mkv" | "avi" | "webm" => "video",
        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tiff" => "image",
        "mp3" | "wav" | "aac" | "m4a" | "flac" | "opus" | "ogg" => "audio",
        _ => "unknown",
    }
}

#[cfg(target_os = "windows")]
fn run_silently(command: &mut Command) -> &mut Command {
    command.creation_flags(CREATE_NO_WINDOW)
}

#[cfg(not(target_os = "windows"))]
fn run_silently(command: &mut Command) -> &mut Command {
    command
}

fn preset_profile(preset_id: &str) -> PresetProfile {
    match preset_id {
        "small" => PresetProfile {
            video_crf: 34,
            video_webm_crf: 40,
            audio_bitrate_kbps: 96,
            image_quality: 58,
        },
        "high-quality" => PresetProfile {
            video_crf: 22,
            video_webm_crf: 28,
            audio_bitrate_kbps: 192,
            image_quality: 88,
        },
        _ => PresetProfile {
            video_crf: 28,
            video_webm_crf: 33,
            audio_bitrate_kbps: 128,
            image_quality: 74,
        },
    }
}

fn format_choice_for_kind(request: &BatchProcessRequest, media_kind: &str) -> String {
    match media_kind {
        "video" => request.video_format.to_lowercase(),
        "image" => request.image_format.to_lowercase(),
        "audio" => request.audio_format.to_lowercase(),
        _ => String::new(),
    }
}

fn build_output_path(
    output_dir: &Path,
    input_path: &Path,
    suffix: &str,
    extension: &str,
) -> Result<PathBuf, String> {
    let stem = input_path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("Could not determine file stem for {}", input_path.display()))?;

    Ok(output_dir.join(format!("{stem}{suffix}.{extension}")))
}

fn ffmpeg_resize_filter(long_edge: Option<u32>) -> Option<String> {
    long_edge.map(|edge| {
        format!("scale='if(gte(iw,ih),min(iw,{edge}),-2)':'if(gte(iw,ih),-2,min(ih,{edge}))'")
    })
}

fn build_video_args(
    input_path: &Path,
    output_path: &Path,
    request: &BatchProcessRequest,
    profile: &PresetProfile,
) -> Result<Vec<String>, String> {
    let target = format_choice_for_kind(request, "video");
    let audio_bitrate = format!("{}k", profile.audio_bitrate_kbps);
    let mut args = vec!["-i".into(), input_path.display().to_string()];

    if let Some(filter) = ffmpeg_resize_filter(request.resize_long_edge) {
        args.push("-vf".into());
        args.push(filter);
    }

    match target.as_str() {
        "webm" => {
            args.extend([
                "-c:v".into(),
                "libvpx-vp9".into(),
                "-b:v".into(),
                "0".into(),
                "-crf".into(),
                profile.video_webm_crf.to_string(),
                "-c:a".into(),
                "libopus".into(),
                "-b:a".into(),
                audio_bitrate,
            ]);
        }
        "mp4" | "mkv" | "mov" => {
            args.extend([
                "-c:v".into(),
                "libx264".into(),
                "-preset".into(),
                "medium".into(),
                "-crf".into(),
                profile.video_crf.to_string(),
                "-c:a".into(),
                "aac".into(),
                "-b:a".into(),
                audio_bitrate,
            ]);
        }
        _ => return Err(format!("Unsupported video target format: {target}")),
    }

    args.push(output_path.display().to_string());
    Ok(args)
}

fn build_audio_args(
    input_path: &Path,
    output_path: &Path,
    request: &BatchProcessRequest,
    profile: &PresetProfile,
) -> Result<Vec<String>, String> {
    let target = format_choice_for_kind(request, "audio");
    let bitrate = format!("{}k", profile.audio_bitrate_kbps);
    let mut args = vec!["-i".into(), input_path.display().to_string(), "-vn".into()];

    match target.as_str() {
        "mp3" => args.extend(["-c:a".into(), "libmp3lame".into(), "-b:a".into(), bitrate]),
        "aac" | "m4a" => args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), bitrate]),
        "opus" => args.extend(["-c:a".into(), "libopus".into(), "-b:a".into(), bitrate]),
        "wav" => args.extend(["-c:a".into(), "pcm_s16le".into()]),
        _ => return Err(format!("Unsupported audio target format: {target}")),
    }

    args.push(output_path.display().to_string());
    Ok(args)
}

fn build_image_args(
    input_path: &Path,
    output_path: &Path,
    request: &BatchProcessRequest,
    profile: &PresetProfile,
) -> Result<Vec<String>, String> {
    let target = format_choice_for_kind(request, "image");
    let mut args = vec![
        "-i".into(),
        input_path.display().to_string(),
        "-frames:v".into(),
        "1".into(),
    ];

    if let Some(filter) = ffmpeg_resize_filter(request.resize_long_edge) {
        args.push("-vf".into());
        args.push(filter);
    }

    match target.as_str() {
        "jpg" | "jpeg" => args.extend([
            "-q:v".into(),
            (31_u8.saturating_sub(profile.image_quality / 3)).to_string(),
        ]),
        "webp" => args.extend([
            "-c:v".into(),
            "libwebp".into(),
            "-quality".into(),
            profile.image_quality.to_string(),
        ]),
        "png" => args.extend(["-compression_level".into(), "9".into()]),
        _ => return Err(format!("Unsupported image target format: {target}")),
    }

    args.push(output_path.display().to_string());
    Ok(args)
}

fn build_gif_args(input_path: &Path, output_path: &Path, gif: &GifOptions) -> Vec<String> {
    let filter = format!(
    "[0:v]fps={fps},scale={width}:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer[gif]",
    fps = gif.fps.max(1),
    width = gif.width.max(160),
  );

    vec![
        "-ss".into(),
        format!("{:.2}", gif.start_seconds.max(0.0)),
        "-t".into(),
        format!("{:.2}", gif.duration_seconds.max(0.5)),
        "-i".into(),
        input_path.display().to_string(),
        "-an".into(),
        "-filter_complex".into(),
        filter,
        "-map".into(),
        "[gif]".into(),
        "-loop".into(),
        "0".into(),
        output_path.display().to_string(),
    ]
}

fn default_job_id(input_path: &str, operation: &str) -> String {
    format!("{operation}::{input_path}")
}

fn bundled_tool_name(tool: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{tool}.exe")
    }

    #[cfg(not(target_os = "windows"))]
    {
        tool.to_string()
    }
}

fn bundled_platform_dir() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }

    #[cfg(target_os = "macos")]
    {
        "macos"
    }

    #[cfg(target_os = "linux")]
    {
        "linux"
    }
}

fn bundled_tool_path_candidates(app: &AppHandle, tool: &str) -> Vec<PathBuf> {
    let bundled_name = bundled_tool_name(tool);
    let relative_path = Path::new("ffmpeg")
        .join(bundled_platform_dir())
        .join(&bundled_name);
    let mut candidates = Vec::new();

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join(&relative_path));
        candidates.push(resource_dir.join("resources").join(&relative_path));
    }

    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(relative_path),
    );

    candidates
}

fn resolve_tool_path(app: &AppHandle, tool: &str) -> PathBuf {
    bundled_tool_path_candidates(app, tool)
        .into_iter()
        .find(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from(tool))
}

fn cancelled_batch_result(
    job_id: String,
    label: Option<String>,
    input_path: String,
    media_kind: String,
    operation: String,
    output_path: Option<String>,
    ffmpeg_args: Vec<String>,
) -> BatchJobResult {
    BatchJobResult {
        job_id,
        label,
        input_path,
        media_kind,
        operation,
        output_path,
        success: false,
        skipped: false,
        cancelled: true,
        ffmpeg_args,
        message: "Cancelled by user.".into(),
    }
}

fn read_meminfo_value(field: &str) -> Option<u64> {
    let contents = fs::read_to_string("/proc/meminfo").ok()?;
    contents.lines().find_map(|line| {
        if !line.starts_with(field) {
            return None;
        }

        let kb_value = line
            .split_whitespace()
            .nth(1)
            .and_then(|value| value.parse::<u64>().ok())?;
        Some(kb_value / 1024)
    })
}

fn available_memory_mb() -> Option<u64> {
    read_meminfo_value("MemAvailable:").or_else(|| read_meminfo_value("MemFree:"))
}

fn total_memory_mb() -> Option<u64> {
    read_meminfo_value("MemTotal:")
}

fn estimate_job_resources(
    job_id: String,
    label: String,
    input_path: &str,
    operation: &str,
) -> ResourceJobEstimate {
    let input = Path::new(input_path);
    let media_kind = detect_media_kind(input).to_string();
    let bytes = fs::metadata(input)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let size_mb = ((bytes as f64) / (1024.0 * 1024.0)).ceil().max(1.0);

    let (base_memory_mb, seconds_factor) = match (media_kind.as_str(), operation) {
        ("video", "compress") => (700.0, 1.25),
        ("video", "convert") => (620.0, 1.05),
        ("video", "gif") => (900.0, 1.6),
        ("image", "compress") => (180.0, 0.18),
        ("image", "convert") => (160.0, 0.12),
        ("audio", "compress") => (120.0, 0.2),
        ("audio", "convert") => (110.0, 0.15),
        _ => (160.0, 0.2),
    };

    let estimated_memory_mb = (base_memory_mb + (size_mb * 0.35)).round() as u64;
    let estimated_seconds = (8.0 + (size_mb * seconds_factor)).round().max(2.0) as u64;

    ResourceJobEstimate {
        job_id,
        label,
        media_kind,
        operation: operation.to_string(),
        estimated_memory_mb,
        estimated_seconds,
    }
}

fn build_resource_jobs(request: &ResourcePlanRequest) -> Vec<ResourceJobEstimate> {
    let jobs = if let Some(mixed_jobs) = &request.mixed_jobs {
        if !mixed_jobs.is_empty() {
            mixed_jobs
                .iter()
                .map(|job| {
                    estimate_job_resources(
                        job.job_id.clone(),
                        job.label
                            .clone()
                            .unwrap_or_else(|| basename_for_path(&job.input_path)),
                        &job.input_path,
                        &job.mode.to_lowercase(),
                    )
                })
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if !jobs.is_empty() {
        return jobs;
    }

    if let Some(mixed_jobs) = &request.mixed_jobs {
        if !mixed_jobs.is_empty() {
            return Vec::new();
        }
    }

    if request.mode.eq_ignore_ascii_case("gif") {
        let gif_segments = request.gif_segments.clone().unwrap_or_default();
        if !gif_segments.is_empty() {
            return gif_segments
                .iter()
                .map(|segment| {
                    estimate_job_resources(
                        segment.job_id.clone(),
                        segment
                            .label
                            .clone()
                            .unwrap_or_else(|| basename_for_path(&segment.input_path)),
                        &segment.input_path,
                        "gif",
                    )
                })
                .collect::<Vec<_>>();
        }
    }

    request
        .input_paths
        .iter()
        .map(|path| {
            estimate_job_resources(
                default_job_id(path, &request.mode.to_lowercase()),
                basename_for_path(path),
                path,
                &request.mode.to_lowercase(),
            )
        })
        .collect::<Vec<_>>()
}

fn basename_for_path(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
        .to_string()
}

#[tauri::command]
pub(crate) fn analyze_resource_plan(request: ResourcePlanRequest) -> ResourcePlan {
    let jobs = build_resource_jobs(&request);
    let logical_cores = thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    let total_memory_mb = total_memory_mb();
    let available_memory_mb = available_memory_mb();
    let requested_parallel_jobs = request.max_parallel_jobs.unwrap_or(1).clamp(1, 1);
    let total_estimated_seconds = jobs.iter().map(|job| job.estimated_seconds).sum::<u64>();
    let max_job_memory_mb = jobs
        .iter()
        .map(|job| job.estimated_memory_mb)
        .max()
        .unwrap_or(0);
    let mut safe_parallel_jobs = requested_parallel_jobs.min(logical_cores.max(1));

    if let Some(available_memory_mb) = available_memory_mb {
        let memory_headroom_mb = ((available_memory_mb as f64) * 0.72).floor() as u64;
        let by_memory = if max_job_memory_mb == 0 {
            1
        } else {
            (memory_headroom_mb / max_job_memory_mb).max(1) as usize
        };
        safe_parallel_jobs = safe_parallel_jobs.min(by_memory.max(1));
    }

    safe_parallel_jobs = safe_parallel_jobs.max(1);
    let effective_parallel_jobs = requested_parallel_jobs.max(1);
    let estimated_parallel_memory_mb =
        max_job_memory_mb.saturating_mul(effective_parallel_jobs as u64);
    let estimated_parallel_seconds =
        ((total_estimated_seconds as f64) / (effective_parallel_jobs as f64)).ceil() as u64;
    let can_run_in_parallel = effective_parallel_jobs <= safe_parallel_jobs;
    let should_use_sequential = !can_run_in_parallel && jobs.len() > 1;

    let summary = if jobs.is_empty() {
        "No jobs selected yet.".to_string()
    } else if should_use_sequential {
        format!(
      "Estimated parallel load is too high for this machine right now. Use sequential mode or lower parallel jobs to {}.",
      safe_parallel_jobs
    )
    } else {
        format!(
            "Estimated batch time is about {} min with up to {} parallel jobs.",
            ((estimated_parallel_seconds as f64) / 60.0).ceil() as u64,
            effective_parallel_jobs
        )
    };

    ResourcePlan {
        logical_cores,
        available_memory_mb,
        total_memory_mb,
        max_parallel_jobs: effective_parallel_jobs,
        safe_parallel_jobs,
        estimated_parallel_memory_mb,
        estimated_total_seconds: total_estimated_seconds,
        estimated_parallel_seconds,
        can_run_in_parallel,
        should_use_sequential,
        summary,
        jobs,
    }
}

fn mixed_job_to_request(job: &MixedJobRequest) -> BatchProcessRequest {
    BatchProcessRequest {
        run_id: format!("mixed::{}", job.job_id),
        input_paths: vec![job.input_path.clone()],
        output_dir: job.output_dir.clone(),
        mode: job.mode.clone(),
        preset_id: job.preset_id.clone(),
        video_format: job.video_format.clone(),
        image_format: job.image_format.clone(),
        audio_format: job.audio_format.clone(),
        resize_long_edge: job.resize_long_edge,
        max_parallel_jobs: Some(1),
        gif: job.gif.clone(),
        gif_segments: None,
        mixed_jobs: None,
        overwrite: job.overwrite,
    }
}

fn emit_batch_progress(app: &AppHandle, event: BatchProgressEvent) {
    let _ = app.emit("batch-progress", event);
}

fn emit_cancelled_progress(
    app: &AppHandle,
    job_id: &str,
    label: Option<&str>,
    input_path: &str,
    media_kind: &str,
    operation: &str,
    output_path: Option<&Path>,
    progress_percent: Option<f64>,
) {
    emit_batch_progress(
        app,
        BatchProgressEvent {
            job_id: job_id.into(),
            label: label.map(str::to_string),
            input_path: input_path.into(),
            media_kind: media_kind.into(),
            operation: operation.into(),
            status: "cancelled".into(),
            progress_percent,
            output_path: output_path.map(|path| path.display().to_string()),
            message: "Cancelled by user.".into(),
            speed: None,
        },
    );
}

fn probe_duration_seconds(app: &AppHandle, input_path: &Path) -> Option<f64> {
    let ffprobe = resolve_tool_path(app, "ffprobe");
    let mut command = Command::new(ffprobe);
    let output = run_silently(
        command
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
            ])
            .arg(input_path),
    )
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<f64>()
        .ok()
}

fn extract_progress_percent(out_time_us: i64, duration_seconds: Option<f64>) -> Option<f64> {
    duration_seconds.and_then(|duration| {
        if duration <= 0.0 {
            None
        } else {
            Some(((out_time_us as f64 / 1_000_000.0) / duration * 100.0).clamp(0.0, 100.0))
        }
    })
}

fn run_ffmpeg(
    app: &AppHandle,
    run_control: &BatchRunControl,
    job_id: &str,
    label: Option<&str>,
    input_path: &str,
    output_path: &Path,
    media_kind: &str,
    operation: &str,
    args: &[String],
    overwrite: bool,
    duration_seconds: Option<f64>,
) -> Result<String, RunFfmpegError> {
    if run_control.is_cancelled() {
        emit_cancelled_progress(
            app,
            job_id,
            label,
            input_path,
            media_kind,
            operation,
            Some(output_path),
            Some(0.0),
        );
        return Err(RunFfmpegError::Cancelled);
    }

    let ffmpeg = resolve_tool_path(app, "ffmpeg");
    let mut command = Command::new(ffmpeg);
    run_silently(&mut command);
    if overwrite {
        command.arg("-y");
    } else {
        command.arg("-n");
    }
    command
        .arg("-progress")
        .arg("pipe:1")
        .arg("-nostats")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|error| RunFfmpegError::Failed(format!("Failed to run ffmpeg: {error}")))?;
    run_control.register_process(job_id, child.id());

    emit_batch_progress(
        app,
        BatchProgressEvent {
            job_id: job_id.into(),
            label: label.map(str::to_string),
            input_path: input_path.into(),
            media_kind: media_kind.into(),
            operation: operation.into(),
            status: "running".into(),
            progress_percent: Some(0.0),
            output_path: Some(output_path.display().to_string()),
            message: "Started ffmpeg job.".into(),
            speed: None,
        },
    );

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| RunFfmpegError::Failed("Failed to capture ffmpeg stderr.".to_string()))?;
    let stderr_handle = thread::spawn(move || {
        let mut stderr = BufReader::new(stderr);
        let mut buffer = String::new();
        let _ = stderr.read_to_string(&mut buffer);
        buffer
    });

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| RunFfmpegError::Failed("Failed to capture ffmpeg stdout.".to_string()))?;
    let reader = BufReader::new(stdout);
    let mut last_out_time_us = 0_i64;
    let mut last_speed: Option<String> = None;
    let mut last_percent = 0.0_f64;

    for line_result in reader.lines() {
        if run_control.is_cancelled() {
            let _ = child.kill();
        }

        let line = line_result.map_err(|error| {
            RunFfmpegError::Failed(format!("Failed to read ffmpeg progress: {error}"))
        })?;
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        match key {
            "out_time_us" => {
                last_out_time_us = value.parse::<i64>().unwrap_or(last_out_time_us);
            }
            "speed" => {
                if !value.is_empty() && value != "N/A" {
                    last_speed = Some(value.to_string());
                }
            }
            "progress" => {
                let progress_percent = extract_progress_percent(last_out_time_us, duration_seconds)
                    .or(Some(last_percent));

                if let Some(percent) = progress_percent {
                    last_percent = percent.max(last_percent);
                }

                if value == "continue" {
                    emit_batch_progress(
                        app,
                        BatchProgressEvent {
                            job_id: job_id.into(),
                            label: label.map(str::to_string),
                            input_path: input_path.into(),
                            media_kind: media_kind.into(),
                            operation: operation.into(),
                            status: "progress".into(),
                            progress_percent,
                            output_path: Some(output_path.display().to_string()),
                            message: "Processing.".into(),
                            speed: last_speed.clone(),
                        },
                    );
                }
            }
            _ => {}
        }
    }

    let status = child
        .wait()
        .map_err(|error| RunFfmpegError::Failed(format!("Failed to wait for ffmpeg: {error}")))?;
    run_control.unregister_process(job_id);
    let stderr = stderr_handle
        .join()
        .map_err(|_| RunFfmpegError::Failed("Failed to join ffmpeg stderr reader.".to_string()))?;
    let stderr = stderr.trim().to_string();

    if run_control.is_cancelled() {
        emit_cancelled_progress(
            app,
            job_id,
            label,
            input_path,
            media_kind,
            operation,
            Some(output_path),
            Some(last_percent),
        );
        return Err(RunFfmpegError::Cancelled);
    }

    if status.success() {
        emit_batch_progress(
            app,
            BatchProgressEvent {
                job_id: job_id.into(),
                label: label.map(str::to_string),
                input_path: input_path.into(),
                media_kind: media_kind.into(),
                operation: operation.into(),
                status: "completed".into(),
                progress_percent: Some(100.0),
                output_path: Some(output_path.display().to_string()),
                message: "Completed successfully.".into(),
                speed: last_speed,
            },
        );
        Ok(if stderr.is_empty() {
            "Completed successfully.".into()
        } else {
            stderr
        })
    } else {
        emit_batch_progress(
            app,
            BatchProgressEvent {
                job_id: job_id.into(),
                label: label.map(str::to_string),
                input_path: input_path.into(),
                media_kind: media_kind.into(),
                operation: operation.into(),
                status: "failed".into(),
                progress_percent: Some(last_percent),
                output_path: Some(output_path.display().to_string()),
                message: if stderr.is_empty() {
                    "ffmpeg exited with a non-zero status.".into()
                } else {
                    stderr.clone()
                },
                speed: last_speed,
            },
        );
        Err(RunFfmpegError::Failed(if stderr.is_empty() {
            "ffmpeg exited with a non-zero status.".into()
        } else {
            stderr
        }))
    }
}

fn process_single_job(
    app: &AppHandle,
    run_control: &BatchRunControl,
    input_path: &str,
    request: &BatchProcessRequest,
    gif_segment: Option<&GifSegmentRequest>,
    job_id_override: Option<&str>,
    label_override: Option<&str>,
) -> BatchJobResult {
    let input = PathBuf::from(input_path);
    let media_kind = detect_media_kind(&input).to_string();
    let mode = request.mode.to_lowercase();
    let overwrite = request.overwrite.unwrap_or(true);
    let profile = preset_profile(&request.preset_id);
    let job_id = job_id_override
        .map(str::to_string)
        .or_else(|| gif_segment.map(|segment| segment.job_id.clone()))
        .unwrap_or_else(|| default_job_id(input_path, &mode));
    let label = label_override
        .map(str::to_string)
        .or_else(|| gif_segment.and_then(|segment| segment.label.clone()));

    if run_control.is_cancelled() {
        emit_cancelled_progress(
            app,
            &job_id,
            label.as_deref(),
            input_path,
            &media_kind,
            &mode,
            None,
            Some(0.0),
        );
        return cancelled_batch_result(
            job_id,
            label,
            input_path.into(),
            media_kind,
            mode,
            None,
            vec![],
        );
    }

    if !input.exists() {
        emit_batch_progress(
            app,
            BatchProgressEvent {
                job_id: job_id.clone(),
                label: label.clone(),
                input_path: input_path.into(),
                media_kind: media_kind.clone(),
                operation: mode.clone(),
                status: "failed".into(),
                progress_percent: Some(0.0),
                output_path: None,
                message: "Input file does not exist.".into(),
                speed: None,
            },
        );
        return BatchJobResult {
            job_id,
            label,
            input_path: input_path.into(),
            media_kind,
            operation: mode,
            output_path: None,
            success: false,
            skipped: false,
            cancelled: false,
            ffmpeg_args: vec![],
            message: "Input file does not exist.".into(),
        };
    }

    if media_kind == "unknown" {
        emit_batch_progress(
            app,
            BatchProgressEvent {
                job_id: job_id.clone(),
                label: label.clone(),
                input_path: input_path.into(),
                media_kind: media_kind.clone(),
                operation: mode.clone(),
                status: "skipped".into(),
                progress_percent: Some(0.0),
                output_path: None,
                message: "Unsupported media type for this batch.".into(),
                speed: None,
            },
        );
        return BatchJobResult {
            job_id,
            label,
            input_path: input_path.into(),
            media_kind,
            operation: mode,
            output_path: None,
            success: false,
            skipped: true,
            cancelled: false,
            ffmpeg_args: vec![],
            message: "Unsupported media type for this batch.".into(),
        };
    }

    let output_dir = PathBuf::from(&request.output_dir);
    if let Err(error) = fs::create_dir_all(&output_dir) {
        let message = format!("Failed to create output directory: {error}");
        emit_batch_progress(
            app,
            BatchProgressEvent {
                job_id: job_id.clone(),
                label: label.clone(),
                input_path: input_path.into(),
                media_kind: media_kind.clone(),
                operation: mode.clone(),
                status: "failed".into(),
                progress_percent: Some(0.0),
                output_path: None,
                message: message.clone(),
                speed: None,
            },
        );
        return BatchJobResult {
            job_id,
            label,
            input_path: input_path.into(),
            media_kind,
            operation: mode,
            output_path: None,
            success: false,
            skipped: false,
            cancelled: false,
            ffmpeg_args: vec![],
            message,
        };
    }

    let (output_path, ffmpeg_args) = if mode == "gif" {
        if media_kind != "video" {
            emit_batch_progress(
                app,
                BatchProgressEvent {
                    job_id: job_id.clone(),
                    label: label.clone(),
                    input_path: input_path.into(),
                    media_kind: media_kind.clone(),
                    operation: mode.clone(),
                    status: "skipped".into(),
                    progress_percent: Some(0.0),
                    output_path: None,
                    message: "GIF generation is only available for video inputs.".into(),
                    speed: None,
                },
            );
            return BatchJobResult {
                job_id,
                label,
                input_path: input_path.into(),
                media_kind,
                operation: mode,
                output_path: None,
                success: false,
                skipped: true,
                cancelled: false,
                ffmpeg_args: vec![],
                message: "GIF generation is only available for video inputs.".into(),
            };
        }

        let gif = gif_segment
            .map(|segment| GifOptions {
                start_seconds: segment.start_seconds,
                duration_seconds: segment.duration_seconds,
                fps: segment.fps,
                width: segment.width,
            })
            .or_else(|| request.gif.clone())
            .unwrap_or(GifOptions {
                start_seconds: 0.0,
                duration_seconds: 3.0,
                fps: 12,
                width: 480,
            });

        let suffix = gif_segment
            .map(|segment| format!("-{}", segment.output_suffix))
            .unwrap_or_else(|| "-gif".to_string());
        let output_path = match build_output_path(&output_dir, &input, &suffix, "gif") {
            Ok(path) => path,
            Err(message) => {
                emit_batch_progress(
                    app,
                    BatchProgressEvent {
                        job_id: job_id.clone(),
                        label: label.clone(),
                        input_path: input_path.into(),
                        media_kind: media_kind.clone(),
                        operation: mode.clone(),
                        status: "failed".into(),
                        progress_percent: Some(0.0),
                        output_path: None,
                        message: message.clone(),
                        speed: None,
                    },
                );
                return BatchJobResult {
                    job_id,
                    label,
                    input_path: input_path.into(),
                    media_kind,
                    operation: mode,
                    output_path: None,
                    success: false,
                    skipped: false,
                    cancelled: false,
                    ffmpeg_args: vec![],
                    message,
                };
            }
        };

        (
            output_path.clone(),
            build_gif_args(&input, &output_path, &gif),
        )
    } else {
        let target = format_choice_for_kind(request, &media_kind);
        let suffix = if mode == "convert" {
            "-converted"
        } else {
            "-compressed"
        };
        let output_path = match build_output_path(&output_dir, &input, suffix, &target) {
            Ok(path) => path,
            Err(message) => {
                emit_batch_progress(
                    app,
                    BatchProgressEvent {
                        job_id: job_id.clone(),
                        label: label.clone(),
                        input_path: input_path.into(),
                        media_kind: media_kind.clone(),
                        operation: mode.clone(),
                        status: "failed".into(),
                        progress_percent: Some(0.0),
                        output_path: None,
                        message: message.clone(),
                        speed: None,
                    },
                );
                return BatchJobResult {
                    job_id,
                    label,
                    input_path: input_path.into(),
                    media_kind,
                    operation: mode,
                    output_path: None,
                    success: false,
                    skipped: false,
                    cancelled: false,
                    ffmpeg_args: vec![],
                    message,
                };
            }
        };

        let args = match media_kind.as_str() {
            "video" => build_video_args(&input, &output_path, request, &profile),
            "audio" => build_audio_args(&input, &output_path, request, &profile),
            "image" => build_image_args(&input, &output_path, request, &profile),
            _ => Err("Unsupported media kind.".into()),
        };

        match args {
            Ok(args) => (output_path, args),
            Err(message) => {
                emit_batch_progress(
                    app,
                    BatchProgressEvent {
                        job_id: job_id.clone(),
                        label: label.clone(),
                        input_path: input_path.into(),
                        media_kind: media_kind.clone(),
                        operation: mode.clone(),
                        status: "failed".into(),
                        progress_percent: Some(0.0),
                        output_path: None,
                        message: message.clone(),
                        speed: None,
                    },
                );
                return BatchJobResult {
                    job_id,
                    label,
                    input_path: input_path.into(),
                    media_kind,
                    operation: mode,
                    output_path: None,
                    success: false,
                    skipped: false,
                    cancelled: false,
                    ffmpeg_args: vec![],
                    message,
                };
            }
        }
    };

    let duration_seconds = if mode == "gif" || media_kind == "video" || media_kind == "audio" {
        probe_duration_seconds(app, &input)
    } else {
        None
    };

    match run_ffmpeg(
        app,
        run_control,
        &job_id,
        label.as_deref(),
        input_path,
        &output_path,
        &media_kind,
        &mode,
        &ffmpeg_args,
        overwrite,
        duration_seconds,
    ) {
        Ok(message) => BatchJobResult {
            job_id,
            label,
            input_path: input_path.into(),
            media_kind,
            operation: mode,
            output_path: Some(output_path.display().to_string()),
            success: true,
            skipped: false,
            cancelled: false,
            ffmpeg_args,
            message,
        },
        Err(RunFfmpegError::Cancelled) => cancelled_batch_result(
            job_id,
            label,
            input_path.into(),
            media_kind,
            mode,
            Some(output_path.display().to_string()),
            ffmpeg_args,
        ),
        Err(RunFfmpegError::Failed(message)) => BatchJobResult {
            job_id,
            label,
            input_path: input_path.into(),
            media_kind,
            operation: mode,
            output_path: Some(output_path.display().to_string()),
            success: false,
            skipped: false,
            cancelled: false,
            ffmpeg_args,
            message,
        },
    }
}

fn process_mixed_job(
    app: &AppHandle,
    run_control: &BatchRunControl,
    job: &MixedJobRequest,
) -> BatchJobResult {
    let request = mixed_job_to_request(job);
    let gif_segment = if job.mode.eq_ignore_ascii_case("gif") {
        let gif = job.gif.clone().unwrap_or(GifOptions {
            start_seconds: 0.0,
            duration_seconds: 3.0,
            fps: 12,
            width: 480,
        });

        Some(GifSegmentRequest {
            job_id: job.job_id.clone(),
            input_path: job.input_path.clone(),
            output_suffix: job
                .output_suffix
                .clone()
                .unwrap_or_else(|| "gif".to_string()),
            label: job.label.clone(),
            start_seconds: gif.start_seconds,
            duration_seconds: gif.duration_seconds,
            fps: gif.fps,
            width: gif.width,
        })
    } else {
        None
    };

    let mut result = process_single_job(
        app,
        run_control,
        &job.input_path,
        &request,
        gif_segment.as_ref(),
        Some(&job.job_id),
        job.label.as_deref(),
    );
    result.job_id = job.job_id.clone();
    if job.label.is_some() {
        result.label = job.label.clone();
    }
    result
}

fn execute_batch(
    app: AppHandle,
    request: BatchProcessRequest,
    run_control: Arc<BatchRunControl>,
) -> Result<BatchProcessResponse, String> {
    let mixed_jobs = request.mixed_jobs.clone().unwrap_or_default();
    let gif_segments = if request.mode.eq_ignore_ascii_case("gif") {
        request.gif_segments.clone().unwrap_or_default()
    } else {
        Vec::new()
    };

    if request.input_paths.is_empty() && gif_segments.is_empty() && mixed_jobs.is_empty() {
        return Ok(BatchProcessResponse { results: vec![] });
    }

    let ffmpeg = resolve_tool_path(&app, "ffmpeg");
    let mut version_command = Command::new(ffmpeg);
    let version_check = run_silently(version_command.arg("-version"))
        .output()
        .map_err(|error| format!("ffmpeg is required but not available: {error}"))?;
    if !version_check.status.success() {
        return Err("ffmpeg is installed but failed to respond correctly.".into());
    }

    let worker_count = request.max_parallel_jobs.unwrap_or(1).clamp(1, 1);
    let queue_items = if !mixed_jobs.is_empty() {
        mixed_jobs
            .iter()
            .enumerate()
            .map(|(index, job)| {
                let estimate = estimate_job_resources(
                    job.job_id.clone(),
                    job.label
                        .clone()
                        .unwrap_or_else(|| basename_for_path(&job.input_path)),
                    &job.input_path,
                    &job.mode.to_lowercase(),
                );
                (
                    index,
                    job.input_path.clone(),
                    None,
                    Some(job.clone()),
                    estimate,
                )
            })
            .collect::<Vec<_>>()
    } else if request.mode.eq_ignore_ascii_case("gif") && !gif_segments.is_empty() {
        gif_segments
            .iter()
            .enumerate()
            .map(|(index, segment)| {
                let estimate = estimate_job_resources(
                    segment.job_id.clone(),
                    segment
                        .label
                        .clone()
                        .unwrap_or_else(|| basename_for_path(&segment.input_path)),
                    &segment.input_path,
                    "gif",
                );
                (
                    index,
                    segment.input_path.clone(),
                    Some(segment.clone()),
                    None,
                    estimate,
                )
            })
            .collect::<Vec<_>>()
    } else {
        request
            .input_paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                let operation = request.mode.to_lowercase();
                let estimate = estimate_job_resources(
                    default_job_id(path, &operation),
                    basename_for_path(path),
                    path,
                    &operation,
                );
                (index, path.clone(), None, None, estimate)
            })
            .collect::<Vec<_>>()
    };

    for (schedule_index, (_, path, gif_segment, mixed_job, _)) in queue_items.iter().enumerate() {
        let (job_id, label, operation, message) = if let Some(job) = mixed_job {
            (
                job.job_id.clone(),
                job.label.clone(),
                job.mode.to_lowercase(),
                format!(
                    "Queued {} job #{}.",
                    job.mode.to_lowercase(),
                    schedule_index + 1
                ),
            )
        } else if let Some(segment) = gif_segment {
            (
                segment.job_id.clone(),
                segment.label.clone(),
                request.mode.to_lowercase(),
                format!("Queued GIF clip #{}.", schedule_index + 1),
            )
        } else {
            let operation = request.mode.to_lowercase();
            (
                default_job_id(path, &operation),
                Some(basename_for_path(path)),
                operation,
                format!("Queued job #{}.", schedule_index + 1),
            )
        };

        emit_batch_progress(
            &app,
            BatchProgressEvent {
                job_id,
                label,
                input_path: path.clone(),
                media_kind: detect_media_kind(Path::new(path)).into(),
                operation,
                status: "queued".into(),
                progress_percent: Some(0.0),
                output_path: None,
                message,
                speed: None,
            },
        );
    }

    let queue = Arc::new(Mutex::new(
        queue_items
            .into_iter()
            .enumerate()
            .map(|(schedule_index, (_, path, gif_segment, mixed_job, _))| {
                (schedule_index, path, gif_segment, mixed_job)
            })
            .collect::<VecDeque<_>>(),
    ));
    let results = Arc::new(Mutex::new(Vec::<(usize, BatchJobResult)>::new()));
    let shared_request = Arc::new(request);
    let mut handles = Vec::new();

    for _ in 0..worker_count {
        let app = app.clone();
        let queue = Arc::clone(&queue);
        let results = Arc::clone(&results);
        let request = Arc::clone(&shared_request);
        let run_control = Arc::clone(&run_control);
        handles.push(thread::spawn(move || loop {
            let next_item = {
                let mut queue = queue.lock().expect("queue mutex poisoned");
                queue.pop_front()
            };

            match next_item {
                Some((index, path, gif_segment, mixed_job)) => {
                    let result = if let Some(mixed_job) = mixed_job.as_ref() {
                        process_mixed_job(&app, &run_control, mixed_job)
                    } else {
                        process_single_job(
                            &app,
                            &run_control,
                            &path,
                            &request,
                            gif_segment.as_ref(),
                            None,
                            None,
                        )
                    };
                    results
                        .lock()
                        .expect("results mutex poisoned")
                        .push((index, result));
                }
                None => break,
            }
        }));
    }

    for handle in handles {
        handle
            .join()
            .map_err(|_| "A batch worker thread panicked.".to_string())?;
    }

    let mut results = results.lock().expect("results mutex poisoned").clone();
    results.sort_by_key(|(index, _)| *index);

    Ok(BatchProcessResponse {
        results: results.into_iter().map(|(_, result)| result).collect(),
    })
}

#[tauri::command]
pub(crate) fn cancel_batch_run(state: State<AppState>, run_id: String) -> Result<(), String> {
    let run_control = {
        let runs = state
            .batch_runs
            .lock()
            .map_err(|_| "Batch run registry is unavailable.".to_string())?;
        runs.get(&run_id)
            .cloned()
            .ok_or_else(|| "No active batch run matches that id.".to_string())?
    };

    run_control.mark_cancelled();
    for pid in run_control.process_ids() {
        let _ = terminate_process(pid);
    }

    Ok(())
}

#[tauri::command]
pub(crate) async fn run_batch_jobs(
    app: AppHandle,
    state: State<'_, AppState>,
    request: BatchProcessRequest,
) -> Result<BatchProcessResponse, String> {
    let run_id = request.run_id.clone();
    let batch_runs = Arc::clone(&state.batch_runs);
    let run_control = Arc::new(BatchRunControl::default());
    {
        let mut runs = batch_runs
            .lock()
            .map_err(|_| "Batch run registry is unavailable.".to_string())?;
        runs.insert(run_id.clone(), Arc::clone(&run_control));
    }

    let result =
        tauri::async_runtime::spawn_blocking(move || execute_batch(app, request, run_control))
            .await
            .map_err(|error| format!("Batch worker join error: {error}"))?;

    if let Ok(mut runs) = batch_runs.lock() {
        runs.remove(&run_id);
    }

    Ok(result?)
}
