use std::path::PathBuf;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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

#[derive(Default)]
pub struct AppState {
    pub tab: crate::ui::Tab,
    pub encode: EncodeJob,
    pub decode: DecodeJob,
    pub log: String,
    pub show_help: Option<crate::ui::HelpTopic>,
}
