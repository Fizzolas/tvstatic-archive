use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct EncodeJob {
    pub input: Option<PathBuf>,
    pub out_frames: Option<PathBuf>,
    pub out_mkv: Option<PathBuf>,
    pub fps: u32,
    pub profile: sllv_core::Profile,
    pub ffmpeg_path: Option<PathBuf>,
    pub rp: sllv_core::RasterParams,
}

impl Default for EncodeJob {
    fn default() -> Self {
        let profile = sllv_core::Profile::Archive;
        let rp = profile.defaults();
        Self {
            input: None,
            out_frames: None,
            out_mkv: None,
            fps: 24,
            profile,
            ffmpeg_path: None,
            rp,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DecodeJob {
    pub input_frames: Option<PathBuf>,
    pub input_mkv: Option<PathBuf>,
    /// When Some, extract the tar into this folder instead of saving a raw .tar.
    pub out_dir: Option<PathBuf>,
    pub profile: sllv_core::Profile,
    pub ffmpeg_path: Option<PathBuf>,
    pub rp: sllv_core::RasterParams,
}

impl Default for DecodeJob {
    fn default() -> Self {
        let profile = sllv_core::Profile::Archive;
        let rp = profile.defaults();
        Self {
            input_frames: None,
            input_mkv: None,
            out_dir: None,
            profile,
            ffmpeg_path: None,
            rp,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Progress {
    pub stage: String,
    pub done: u64,
    pub total: u64,
    pub started_at: Instant,
}

impl Progress {
    /// Returns a human-readable ETA string such as "2m 34s" or "45s".
    pub fn eta_human(&self) -> Option<String> {
        if self.done == 0 || self.total == 0 {
            return None;
        }
        let elapsed = self.started_at.elapsed().as_secs_f64();
        let per_item = elapsed / (self.done as f64);
        let remaining_secs = (self.total.saturating_sub(self.done) as f64 * per_item).round() as u64;
        if remaining_secs >= 60 {
            Some(format!("{}m {}s", remaining_secs / 60, remaining_secs % 60))
        } else {
            Some(format!("{}s", remaining_secs))
        }
    }
}

/// Maximum number of characters kept in the log before old lines are trimmed.
pub const LOG_MAX_CHARS: usize = 32_768;

/// Append `msg` to `log`, trimming the oldest content when the cap is exceeded.
pub fn log_append(log: &mut String, msg: &str) {
    log.push_str(msg);
    if log.len() > LOG_MAX_CHARS {
        // Drop the first ~quarter so we don't trim on every message.
        let drop_to = log.len() - (LOG_MAX_CHARS * 3 / 4);
        // Advance to the next newline so we don't split mid-line.
        let drop_to = log[drop_to..]
            .find('\n')
            .map(|i| drop_to + i + 1)
            .unwrap_or(drop_to);
        *log = format!("[...older log trimmed...]\n{}", &log[drop_to..]);
    }
}

pub struct AppState {
    pub tab: crate::ui::Tab,
    pub encode: EncodeJob,
    pub decode: DecodeJob,
    pub log: String,
    pub show_help: Option<crate::ui::HelpTopic>,
    pub is_running: bool,
    pub progress: Option<Progress>,
    pub progress_rx: Option<mpsc::Receiver<sllv_core::raster::ProgressMsg>>,
    /// Set to true by the UI cancel button; checked by the worker thread.
    pub cancel_flag: Arc<AtomicBool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tab: crate::ui::Tab::Encode,
            encode: EncodeJob::default(),
            decode: DecodeJob::default(),
            log: String::new(),
            show_help: None,
            is_running: false,
            progress: None,
            progress_rx: None,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl AppState {
    /// Signal the running worker thread to stop and reset running state.
    pub fn request_cancel(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }

    /// Reset the cancel flag and prepare for a new job.
    pub fn begin_job(&mut self) {
        self.cancel_flag.store(false, Ordering::Relaxed);
        self.is_running = true;
        self.progress = Some(Progress {
            stage: "starting".into(),
            done: 0,
            total: 1,
            started_at: std::time::Instant::now(),
        });
    }
}
