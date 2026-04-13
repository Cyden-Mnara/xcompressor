//! Batch run cancellation and child-process tracking.
//!
//! The Tauri command layer starts FFmpeg jobs on worker threads. This module
//! keeps the shared cancellation flag and the OS process IDs that need to be
//! terminated when a user cancels a batch or closes the app window.

use std::{
    collections::HashMap,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

/// Shared state for one active batch run.
///
/// Each FFmpeg child process is registered while it is running so cancellation
/// can terminate work that is already outside Rust's direct control.
#[derive(Default)]
pub(crate) struct BatchRunControl {
    cancelled: AtomicBool,
    processes: Mutex<HashMap<String, u32>>,
}

impl BatchRunControl {
    /// Returns whether the current run has received a cancellation request.
    pub(crate) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Marks the current run as cancelled.
    pub(crate) fn mark_cancelled(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Records the OS process ID for a running job.
    pub(crate) fn register_process(&self, job_id: &str, pid: u32) {
        if let Ok(mut processes) = self.processes.lock() {
            processes.insert(job_id.to_string(), pid);
        }
    }

    /// Removes the OS process ID for a job that has exited.
    pub(crate) fn unregister_process(&self, job_id: &str) {
        if let Ok(mut processes) = self.processes.lock() {
            processes.remove(job_id);
        }
    }

    /// Returns a snapshot of all currently running child process IDs.
    pub(crate) fn process_ids(&self) -> Vec<u32> {
        self.processes
            .lock()
            .map(|processes| processes.values().copied().collect())
            .unwrap_or_default()
    }
}

/// Requests cancellation for all registered runs and terminates active FFmpeg processes.
pub(crate) fn cancel_active_batch_runs(
    batch_runs: &Arc<Mutex<HashMap<String, Arc<BatchRunControl>>>>,
) {
    let controls = batch_runs
        .lock()
        .map(|runs| runs.values().cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    for control in controls {
        control.mark_cancelled();
        for pid in control.process_ids() {
            let _ = terminate_process(pid);
        }
    }
}

/// Terminates one child process using the platform's native process tool.
pub(crate) fn terminate_process(pid: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()
            .map_err(|error| format!("Failed to terminate ffmpeg process {pid}: {error}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("taskkill failed for ffmpeg process {pid}."))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let term_status = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .map_err(|error| format!("Failed to terminate ffmpeg process {pid}: {error}"))?;

        thread::sleep(Duration::from_millis(750));

        let kill_status = Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .status()
            .map_err(|error| format!("Failed to force-kill ffmpeg process {pid}: {error}"))?;

        if term_status.success() || kill_status.success() {
            Ok(())
        } else {
            Err(format!("kill failed for ffmpeg process {pid}."))
        }
    }
}
