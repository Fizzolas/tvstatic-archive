use anyhow::{bail, Context};
use std::path::Path;
use std::process::Command;

/// Convert a frames directory to Matroska/FFV1 via ffmpeg.
///
/// Uses image2 demuxer with numbered frame filenames `frame_%06d.png`.
/// FFV1 is recommended in Matroska for archival use, and FFmpeg documents common encode flags.
/// See: Encode/FFV1 wiki. [web:60]
pub fn frames_to_ffv1_mkv(frames_dir: &Path, out_mkv: &Path, fps: u32) -> anyhow::Result<()> {
    let input_pattern = frames_dir.join("frame_%06d.png");

    // Use FFV1 with level 3 (widely recommended for production).
    // Keep pix_fmt rgb24 since our frames are synthetic RGB.
    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-f")
        .arg("image2")
        .arg("-framerate")
        .arg(format!("{fps}"))
        .arg("-i")
        .arg(input_pattern)
        .arg("-c:v")
        .arg("ffv1")
        .arg("-level")
        .arg("3")
        .arg("-pix_fmt")
        .arg("rgb24")
        .arg(out_mkv)
        .status()
        .context("spawn ffmpeg")?;

    if !status.success() {
        bail!("ffmpeg failed: {status}");
    }

    Ok(())
}
