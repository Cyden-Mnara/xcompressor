//! Live system metrics command.

use tauri::State;

use crate::{AppState, LiveSystemMetrics};

#[tauri::command]
pub(crate) fn get_live_system_metrics(state: State<AppState>) -> LiveSystemMetrics {
    let mut system = state.system.lock().expect("system metrics mutex poisoned");
    system.refresh_cpu_usage();
    system.refresh_memory();

    LiveSystemMetrics {
        cpu_usage_percent: system.global_cpu_usage(),
        used_memory_mb: system.used_memory() / (1024 * 1024),
        available_memory_mb: system.available_memory() / (1024 * 1024),
        total_memory_mb: system.total_memory() / (1024 * 1024),
    }
}
