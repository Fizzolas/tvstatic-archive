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

/// Extract `frame_%06d.png` into `out_frames_dir` from a video file.
///
/// Uses ffmpeg `-vsync 0` to avoid frame duplication and `-start_number 0` to match our naming.
/// The ffmpeg docs describe `-vsync` behavior and image output patterns. [web:173]
pub fn mkv_to_frames(in_video: &Path, out_frames_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(out_frames_dir).context("create out frames dir")?;

    let out_pattern = out_frames_dir.join("frame_%06d.png");

    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(in_video)
        .arg("-vsync")
        .arg("0")
        .arg("-start_number")
        .arg("0")
        .arg(out_pattern)
        .status()
        .context("spawn ffmpeg")?;

    if !status.success() {
        bail!("ffmpeg failed: {status}");
    }

    Ok(())
}
