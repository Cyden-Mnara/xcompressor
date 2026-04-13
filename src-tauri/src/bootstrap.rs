//! Static bootstrap data and compression planning helpers.
//!
//! These commands provide the frontend with supported media modes, output
//! targets, preset summaries, and lightweight preset recommendations.

use crate::{
    AppBootstrap, CompressionPlan, CompressionPlanRequest, CompressionPreset, FormatTargets,
    MediaCapability,
};

fn bootstrap_data() -> AppBootstrap {
    AppBootstrap {
        app_name: "xcompressor",
        version: env!("CARGO_PKG_VERSION"),
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
                compression_modes: vec![
                    "bitrate reduction",
                    "sample-rate reduction",
                    "voice optimized",
                ],
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
pub(crate) fn get_app_bootstrap() -> AppBootstrap {
    bootstrap_data()
}

#[tauri::command]
pub(crate) fn plan_compression(request: CompressionPlanRequest) -> CompressionPlan {
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
