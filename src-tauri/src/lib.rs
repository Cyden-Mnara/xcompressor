use serde::{Deserialize, Serialize};
use std::{
  collections::VecDeque,
  fs,
  io::{BufRead, BufReader, Read},
  path::{Path, PathBuf},
  process::{Command, Stdio},
  sync::{Arc, Mutex},
  thread,
};
use sysinfo::System;
use tauri::{AppHandle, Emitter};

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

#[derive(Debug)]
struct PresetProfile {
  video_crf: u8,
  video_webm_crf: u8,
  audio_bitrate_kbps: u16,
  image_quality: u8,
}

fn bootstrap_data() -> AppBootstrap {
  AppBootstrap {
    app_name: "xcompressor",
    summary: "Batch multimedia compression, format conversion, and GIF generation.",
    presets: vec![
      CompressionPreset {
        id: "small",
        label: "Small",
        description: "Aggressive compression for lightweight sharing.",
        quality_range: "45-60%",
        size_reduction_range: "55-75%",
      },
      CompressionPreset {
        id: "balanced",
        label: "Balanced",
        description: "Default preset with good visual and audio retention.",
        quality_range: "60-78%",
        size_reduction_range: "30-50%",
      },
      CompressionPreset {
        id: "high-quality",
        label: "High Quality",
        description: "Safer range for archive-ready exports.",
        quality_range: "78-92%",
        size_reduction_range: "10-25%",
      },
    ],
    media_capabilities: vec![
      MediaCapability {
        kind: "video",
        compression_modes: vec!["CRF compression", "bitrate target", "resize + compress"],
        conversions: vec!["mp4 -> webm", "mov -> mp4", "mkv -> mp4"],
      },
      MediaCapability {
        kind: "image",
        compression_modes: vec!["lossy", "lossless", "resize + optimize"],
        conversions: vec!["png -> webp", "jpg -> webp", "webp -> jpg"],
      },
      MediaCapability {
        kind: "audio",
        compression_modes: vec!["bitrate reduction", "sample-rate reduction", "voice optimized"],
        conversions: vec!["wav -> mp3", "flac -> aac", "m4a -> mp3"],
      },
    ],
    format_targets: vec![
      FormatTargets {
        kind: "video",
        targets: vec!["mp4", "webm", "mkv"],
      },
      FormatTargets {
        kind: "image",
        targets: vec!["webp", "jpg", "png"],
      },
      FormatTargets {
        kind: "audio",
        targets: vec!["mp3", "aac", "wav", "opus"],
      },
    ],
    gif_workflow: vec![
      "Pick a video clip and trim a start/end range.",
      "Choose frame rate, width, and quality target.",
      "Export as GIF or MP4 preview.",
    ],
  }
}

#[tauri::command]
fn open_media_in_system_player(path: String) -> Result<(), String> {
  let input = PathBuf::from(&path);
  if !input.exists() {
    return Err("Video file does not exist.".into());
  }

  #[cfg(target_os = "linux")]
  let mut command = {
    let mut command = Command::new("xdg-open");
    command.arg(&path);
    command
  };

  #[cfg(target_os = "macos")]
  let mut command = {
    let mut command = Command::new("open");
    command.arg(&path);
    command
  };

  #[cfg(target_os = "windows")]
  let mut command = {
    let mut command = Command::new("cmd");
    command.args(["/C", "start", "", &path]);
    command
  };

  command
    .spawn()
    .map_err(|error| format!("Failed to open system video player: {error}"))?;

  Ok(())
}

#[tauri::command]
fn get_app_bootstrap() -> AppBootstrap {
  bootstrap_data()
}

#[tauri::command]
fn get_live_system_metrics() -> LiveSystemMetrics {
  let mut system = System::new_all();
  system.refresh_cpu_all();
  system.refresh_memory();
  std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
  system.refresh_cpu_usage();
  system.refresh_memory();

  LiveSystemMetrics {
    cpu_usage_percent: system.global_cpu_usage(),
    used_memory_mb: system.used_memory() / (1024 * 1024),
    available_memory_mb: system.available_memory() / (1024 * 1024),
    total_memory_mb: system.total_memory() / (1024 * 1024),
  }
}

#[tauri::command]
fn plan_compression(request: CompressionPlanRequest) -> CompressionPlan {
  let media_kind = request.media_kind.to_lowercase();
  let goal = request.goal.to_lowercase();

  let (recommended_preset, estimated_output_range, notes) =
    match (media_kind.as_str(), goal.as_str()) {
      ("video", "smallest") => (
        "small",
        "20-45% of original size",
        vec![
          "Use H.265 or VP9 for stronger savings.",
          "Prefer CRF 30-34 for screeners and previews.",
        ],
      ),
      ("image", "smallest") => (
        "small",
        "25-50% of original size",
        vec![
          "WebP and AVIF usually outperform JPG and PNG.",
          "Resize oversized inputs before final encoding.",
        ],
      ),
      ("audio", "smallest") => (
        "small",
        "15-35% of original size",
        vec![
          "Speech can usually drop to 48-64 kbps mono.",
          "Music is safer around 96-128 kbps AAC or MP3.",
        ],
      ),
      (_, "archive") => (
        "high-quality",
        "75-90% of original size",
        vec![
          "Keep dimensions and sample rate unchanged.",
          "Favor visually lossless settings over maximum savings.",
        ],
      ),
      _ => (
        "balanced",
        "50-70% of original size",
        vec![
          "Start with the balanced preset before tuning deeper codec settings.",
          "Preview a short sample clip before batch-processing an entire folder.",
        ],
      ),
    };

  CompressionPlan {
    media_kind,
    goal,
    recommended_preset,
    estimated_output_range,
    notes,
  }
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
    format!(
      "scale='if(gte(iw,ih),min(iw,{edge}),-2)':'if(gte(iw,ih),-2,min(ih,{edge}))'"
    )
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
  let mut args = vec![
    "-i".into(),
    input_path.display().to_string(),
  ];

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
  let mut args = vec![
    "-i".into(),
    input_path.display().to_string(),
    "-vn".into(),
  ];

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
    "jpg" | "jpeg" => args.extend(["-q:v".into(), (31_u8.saturating_sub(profile.image_quality / 3)).to_string()]),
    "webp" => args.extend(["-c:v".into(), "libwebp".into(), "-quality".into(), profile.image_quality.to_string()]),
    "png" => args.extend(["-compression_level".into(), "9".into()]),
    _ => return Err(format!("Unsupported image target format: {target}")),
  }

  args.push(output_path.display().to_string());
  Ok(args)
}

fn build_gif_args(
  input_path: &Path,
  output_path: &Path,
  gif: &GifOptions,
) -> Vec<String> {
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
  let bytes = fs::metadata(input).map(|metadata| metadata.len()).unwrap_or(0);
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
  if let Some(mixed_jobs) = &request.mixed_jobs {
    if !mixed_jobs.is_empty() {
      return mixed_jobs
        .iter()
        .map(|job| {
          estimate_job_resources(
            job.job_id.clone(),
            job.label.clone().unwrap_or_else(|| basename_for_path(&job.input_path)),
            &job.input_path,
            &job.mode.to_lowercase(),
          )
        })
        .collect();
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
        .collect();
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
    .collect()
}

fn basename_for_path(path: &str) -> String {
  Path::new(path)
    .file_name()
    .and_then(|value| value.to_str())
    .unwrap_or(path)
    .to_string()
}

#[tauri::command]
fn analyze_resource_plan(request: ResourcePlanRequest) -> ResourcePlan {
  let jobs = build_resource_jobs(&request);
  let logical_cores = thread::available_parallelism()
    .map(|count| count.get())
    .unwrap_or(1);
  let total_memory_mb = total_memory_mb();
  let available_memory_mb = available_memory_mb();
  let requested_parallel_jobs = request.max_parallel_jobs.unwrap_or(2).clamp(1, 8);
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

fn probe_duration_seconds(input_path: &Path) -> Option<f64> {
  let output = Command::new("ffprobe")
    .args([
      "-v",
      "error",
      "-show_entries",
      "format=duration",
      "-of",
      "default=noprint_wrappers=1:nokey=1",
    ])
    .arg(input_path)
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
  job_id: &str,
  label: Option<&str>,
  input_path: &str,
  output_path: &Path,
  media_kind: &str,
  operation: &str,
  args: &[String],
  overwrite: bool,
  duration_seconds: Option<f64>,
) -> Result<String, String> {
  let mut command = Command::new("ffmpeg");
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
    .map_err(|error| format!("Failed to run ffmpeg: {error}"))?;

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
    .ok_or_else(|| "Failed to capture ffmpeg stderr.".to_string())?;
  let stderr_handle = thread::spawn(move || {
    let mut stderr = BufReader::new(stderr);
    let mut buffer = String::new();
    let _ = stderr.read_to_string(&mut buffer);
    buffer
  });

  let stdout = child
    .stdout
    .take()
    .ok_or_else(|| "Failed to capture ffmpeg stdout.".to_string())?;
  let reader = BufReader::new(stdout);
  let mut last_out_time_us = 0_i64;
  let mut last_speed: Option<String> = None;
  let mut last_percent = 0.0_f64;

  for line_result in reader.lines() {
    let line = line_result.map_err(|error| format!("Failed to read ffmpeg progress: {error}"))?;
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
        let progress_percent =
          extract_progress_percent(last_out_time_us, duration_seconds).or(Some(last_percent));

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
              message: "FFmpeg is processing this file.".into(),
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
    .map_err(|error| format!("Failed to wait for ffmpeg: {error}"))?;
  let stderr = stderr_handle
    .join()
    .map_err(|_| "Failed to join ffmpeg stderr reader.".to_string())?;
  let stderr = stderr.trim().to_string();

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
    Err(if stderr.is_empty() {
      "ffmpeg exited with a non-zero status.".into()
    } else {
      stderr
    })
  }
}

fn process_single_job(
  app: &AppHandle,
  input_path: &str,
  request: &BatchProcessRequest,
  gif_segment: Option<&GifSegmentRequest>,
) -> BatchJobResult {
  let input = PathBuf::from(input_path);
  let media_kind = detect_media_kind(&input).to_string();
  let mode = request.mode.to_lowercase();
  let overwrite = request.overwrite.unwrap_or(true);
  let profile = preset_profile(&request.preset_id);
  let job_id = gif_segment
    .map(|segment| segment.job_id.clone())
    .unwrap_or_else(|| default_job_id(input_path, &mode));
  let label = gif_segment.and_then(|segment| segment.label.clone());

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
          ffmpeg_args: vec![],
          message,
        };
      }
    };

    (output_path.clone(), build_gif_args(&input, &output_path, &gif))
  } else {
    let target = format_choice_for_kind(request, &media_kind);
    let suffix = if mode == "convert" { "-converted" } else { "-compressed" };
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
          ffmpeg_args: vec![],
          message,
        };
      }
    }
  };

  let duration_seconds = if mode == "gif" || media_kind == "video" || media_kind == "audio" {
    probe_duration_seconds(&input)
  } else {
    None
  };

  match run_ffmpeg(
    app,
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
      ffmpeg_args,
      message,
    },
    Err(message) => BatchJobResult {
      job_id,
      label,
      input_path: input_path.into(),
      media_kind,
      operation: mode,
      output_path: Some(output_path.display().to_string()),
      success: false,
      skipped: false,
      ffmpeg_args,
      message,
    },
  }
}

fn process_mixed_job(app: &AppHandle, job: &MixedJobRequest) -> BatchJobResult {
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

  let mut result = process_single_job(app, &job.input_path, &request, gif_segment.as_ref());
  result.job_id = job.job_id.clone();
  if job.label.is_some() {
    result.label = job.label.clone();
  }
  result
}

fn execute_batch(app: AppHandle, request: BatchProcessRequest) -> Result<BatchProcessResponse, String> {
  let mixed_jobs = request.mixed_jobs.clone().unwrap_or_default();
  let gif_segments = if request.mode.eq_ignore_ascii_case("gif") {
    request.gif_segments.clone().unwrap_or_default()
  } else {
    Vec::new()
  };

  if request.input_paths.is_empty() && gif_segments.is_empty() && mixed_jobs.is_empty() {
    return Ok(BatchProcessResponse { results: vec![] });
  }

  let version_check = Command::new("ffmpeg")
    .arg("-version")
    .output()
    .map_err(|error| format!("ffmpeg is required but not available: {error}"))?;
  if !version_check.status.success() {
    return Err("ffmpeg is installed but failed to respond correctly.".into());
  }

  let worker_count = request.max_parallel_jobs.unwrap_or(2).clamp(1, 8);
  if !mixed_jobs.is_empty() {
    for job in &mixed_jobs {
      emit_batch_progress(
        &app,
        BatchProgressEvent {
          job_id: job.job_id.clone(),
          label: job.label.clone(),
          input_path: job.input_path.clone(),
          media_kind: detect_media_kind(Path::new(&job.input_path)).into(),
          operation: job.mode.to_lowercase(),
          status: "queued".into(),
          progress_percent: Some(0.0),
          output_path: None,
          message: format!("Queued {} job.", job.mode.to_lowercase()),
          speed: None,
        },
      );
    }
  } else if request.mode.eq_ignore_ascii_case("gif") && !gif_segments.is_empty() {
    for segment in &gif_segments {
      emit_batch_progress(
        &app,
        BatchProgressEvent {
          job_id: segment.job_id.clone(),
          label: segment.label.clone(),
          input_path: segment.input_path.clone(),
          media_kind: detect_media_kind(Path::new(&segment.input_path)).into(),
          operation: request.mode.to_lowercase(),
          status: "queued".into(),
          progress_percent: Some(0.0),
          output_path: None,
          message: "Queued GIF clip for batch processing.".into(),
          speed: None,
        },
      );
    }
  } else {
    for path in &request.input_paths {
      emit_batch_progress(
        &app,
        BatchProgressEvent {
          job_id: default_job_id(path, &request.mode.to_lowercase()),
          label: Some(
            Path::new(path)
              .file_name()
              .and_then(|value| value.to_str())
              .unwrap_or(path)
              .to_string(),
          ),
          input_path: path.clone(),
          media_kind: detect_media_kind(Path::new(path)).into(),
          operation: request.mode.to_lowercase(),
          status: "queued".into(),
          progress_percent: Some(0.0),
          output_path: None,
          message: "Queued for batch processing.".into(),
          speed: None,
        },
      );
    }
  }
  let queue = Arc::new(Mutex::new(
    if !mixed_jobs.is_empty() {
      mixed_jobs
        .iter()
        .enumerate()
        .map(|(index, job)| (index, job.input_path.clone(), None, Some(job.clone())))
        .collect::<VecDeque<_>>()
    } else if request.mode.eq_ignore_ascii_case("gif") && !gif_segments.is_empty() {
      gif_segments
        .iter()
        .enumerate()
        .map(|(index, segment)| (index, segment.input_path.clone(), Some(segment.clone()), None))
        .collect::<VecDeque<_>>()
    } else {
      request
        .input_paths
        .iter()
        .enumerate()
        .map(|(index, path)| (index, path.clone(), None, None))
        .collect::<VecDeque<_>>()
    },
  ));
  let results = Arc::new(Mutex::new(Vec::<(usize, BatchJobResult)>::new()));
  let shared_request = Arc::new(request);
  let mut handles = Vec::new();

  for _ in 0..worker_count {
    let app = app.clone();
    let queue = Arc::clone(&queue);
    let results = Arc::clone(&results);
    let request = Arc::clone(&shared_request);
    handles.push(thread::spawn(move || loop {
      let next_item = {
        let mut queue = queue.lock().expect("queue mutex poisoned");
        queue.pop_front()
      };

      match next_item {
        Some((index, path, gif_segment, mixed_job)) => {
          let result = if let Some(mixed_job) = mixed_job.as_ref() {
            process_mixed_job(&app, mixed_job)
          } else {
            process_single_job(&app, &path, &request, gif_segment.as_ref())
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
async fn run_batch_jobs(app: AppHandle, request: BatchProcessRequest) -> Result<BatchProcessResponse, String> {
  tauri::async_runtime::spawn_blocking(move || execute_batch(app, request))
    .await
    .map_err(|error| format!("Batch worker join error: {error}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![
      get_app_bootstrap,
      get_live_system_metrics,
      plan_compression,
      analyze_resource_plan,
      open_media_in_system_player,
      run_batch_jobs
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
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
