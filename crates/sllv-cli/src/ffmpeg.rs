use anyhow::{bail, Context};
use std::path::{Path, PathBuf};
use std::process::Command;

fn probe_ffmpeg(ffmpeg: &Path) -> anyhow::Result<()> {
    let status = Command::new(ffmpeg)
        .arg("-version")
        .status()
        .context("spawn ffmpeg -version")?;
    if !status.success() {
        bail!("ffmpeg exists but failed to run: {status}");
    }
    Ok(())
}

fn resolve_ffmpeg(ffmpeg_path: Option<&Path>) -> anyhow::Result<PathBuf> {
    if let Some(p) = ffmpeg_path {
        probe_ffmpeg(p)?;
        return Ok(p.to_path_buf());
    }

    // Default: rely on PATH.
    let p = PathBuf::from("ffmpeg");
    probe_ffmpeg(&p).map(|_| p).map_err(|e| {
        anyhow::anyhow!(
            "FFmpeg not found. Install ffmpeg or pass --ffmpeg-path. Underlying error: {e}"
        )
    })
}

/// Convert a frames directory to Matroska/FFV1 via ffmpeg.
///
/// Uses image2 demuxer with numbered frame filenames `frame_%06d.png`.
/// FFV1 in Matroska is commonly used for lossless/archival workflows. [web:60]
pub fn frames_to_ffv1_mkv(
    frames_dir: &Path,
    out_mkv: &Path,
    fps: u32,
    ffmpeg_path: Option<&Path>,
) -> anyhow::Result<()> {
    let ffmpeg = resolve_ffmpeg(ffmpeg_path)?;
    let input_pattern = frames_dir.join("frame_%06d.png");

    let status = Command::new(ffmpeg)
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
/// Uses ffmpeg `-vsync 0` to avoid frame duplication and `-start_number 0` to match our naming. [web:173]
pub fn mkv_to_frames(
    in_video: &Path,
    out_frames_dir: &Path,
    ffmpeg_path: Option<&Path>,
) -> anyhow::Result<()> {
    let ffmpeg = resolve_ffmpeg(ffmpeg_path)?;

    std::fs::create_dir_all(out_frames_dir).context("create out frames dir")?;
    let out_pattern = out_frames_dir.join("frame_%06d.png");

    let status = Command::new(ffmpeg)
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
