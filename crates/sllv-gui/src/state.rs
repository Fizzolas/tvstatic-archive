use std::path::PathBuf;
use std::sync::mpsc;
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
    pub out_tar: Option<PathBuf>,
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
            out_tar: None,
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
    pub fn eta_secs(&self) -> Option<u64> {
        if self.done == 0 || self.total == 0 {
            return None;
        }
        let elapsed = self.started_at.elapsed().as_secs_f64();
        let per_item = elapsed / (self.done as f64);
        let remaining = self.total.saturating_sub(self.done) as f64;
        Some((remaining * per_item).round() as u64)
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
        }
    }
}
