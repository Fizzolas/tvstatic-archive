use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Encode,
    Decode,
    Doctor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpTopic {
    Profile,
    CellPx,
    BorderCells,
    FiducialSize,
    Deskew,
    Fec,
    Ffmpeg,
    Fps,
}

impl HelpTopic {
    pub fn title(&self) -> &'static str {
        match self {
            HelpTopic::Profile => "Profile",
            HelpTopic::CellPx => "Cell size (px)",
            HelpTopic::BorderCells => "Border cells",
            HelpTopic::FiducialSize => "Fiducial size (cells)",
            HelpTopic::Deskew => "Deskew",
            HelpTopic::Fec => "Error correction (FEC)",
            HelpTopic::Ffmpeg => "FFmpeg path",
            HelpTopic::Fps => "FPS",
        }
    }

    pub fn body(&self) -> &'static str {
        match self {
            HelpTopic::Profile => "Choose Archive for clean, exact frames and optional lossless MKV output. Choose Scan for phone/camera capture (bigger cells + redundancy).",
            HelpTopic::CellPx => "How many screen pixels each data cell uses. Larger values are easier for cameras but produce bigger frames. Keep this consistent between encode and decode.",
            HelpTopic::BorderCells => "Padding around the grid. Helps decoding by giving the detector room to find the content.",
            HelpTopic::FiducialSize => "Size of the corner markers used for locating the frame. Larger can improve camera robustness but increases overhead.",
            HelpTopic::Deskew => "If enabled, the decoder will try to correct perspective/rotation. Recommended for Scan (phone capture).",
            HelpTopic::Fec => "Forward error correction helps recover data when frames are missing or damaged. Recommended for Scan. Avoid changing FEC settings after encoding.",
            HelpTopic::Ffmpeg => "Only needed when you create or decode MKV. If ffmpeg isn't on PATH, select the ffmpeg.exe location here.",
            HelpTopic::Fps => "Frames-per-second used only when writing MKV from images. Does not affect decoding from frames.",
        }
    }
}

pub fn help_button(ui: &mut egui::Ui, state: &mut crate::state::AppState, topic: HelpTopic) {
    if ui.small_button("?").clicked() {
        state.show_help = Some(topic);
    }
}
