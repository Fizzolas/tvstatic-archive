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

    let p = PathBuf::from("ffmpeg");
    probe_ffmpeg(&p).map(|_| p).map_err(|e| {
        anyhow::anyhow!(
            "FFmpeg not found. Install ffmpeg or set an FFmpeg path. Underlying error: {e}"
        )
    })
}

/// Convert a frames directory to Matroska/FFV1 via ffmpeg.
///
/// Uses image2 demuxer with numbered frame filenames `frame_%06d.png`.
/// FFV1 is a lossless codec — pixel values are preserved exactly.
pub fn frames_to_ffv1_mkv(
    frames_dir: &Path,
    out_mkv: &Path,
    fps: u32,
    ffmpeg_path: Option<&Path>,
) -> anyhow::Result<()> {
    let ffmpeg = resolve_ffmpeg(ffmpeg_path)?;
    let input_pattern = frames_dir.join("frame_%06d.png");

    let status = Command::new(&ffmpeg)
        .arg("-y")
        .arg("-f").arg("image2")
        .arg("-framerate").arg(format!("{fps}"))
        .arg("-i").arg(&input_pattern)
        .arg("-c:v").arg("ffv1")
        .arg("-level").arg("3")
        .arg("-pix_fmt").arg("rgb24")
        .arg(out_mkv)
        .status()
        .context("spawn ffmpeg (encode)")?;

    if !status.success() {
        bail!("ffmpeg encode failed: {status}");
    }

    Ok(())
}

/// Extract `frame_%06d.png` into `out_frames_dir` from a video file.
///
/// Requires the input to be a lossless FFV1 stream. Any lossy codec (H.264,
/// H.265, VP9, etc.) silently corrupts colour values through chroma-subsampling
/// and DCT quantisation, which destroys the palette signal. We probe the codec
/// name before extracting and bail out with a clear error if it is not FFV1.
///
/// `-pix_fmt rgb24` is forced on the output side to guarantee 8-bit-per-channel
/// PNG frames regardless of the container's stored pixel format.
pub fn mkv_to_frames(
    in_video: &Path,
    out_frames_dir: &Path,
    ffmpeg_path: Option<&Path>,
) -> anyhow::Result<()> {
    let ffmpeg = resolve_ffmpeg(ffmpeg_path)?;

    // ── Codec guard ─────────────────────────────────────────────────────────
    // Use ffprobe (co-installed with ffmpeg) to read the codec name.
    // If ffprobe is not available we skip the check rather than hard-fail,
    // because some distributions ship ffmpeg without ffprobe.
    let ffprobe = ffmpeg.with_file_name("ffprobe");
    if ffprobe.exists() {
        let probe_out = Command::new(&ffprobe)
            .args(["-v", "error",
                   "-select_streams", "v:0",
                   "-show_entries", "stream=codec_name",
                   "-of", "default=noprint_wrappers=1:nokey=1"])
            .arg(in_video)
            .output()
            .context("spawn ffprobe")?;

        let codec = String::from_utf8_lossy(&probe_out.stdout);
        let codec = codec.trim();
        if !codec.is_empty() && codec != "ffv1" {
            bail!(
                "Input video codec is '{}', not 'ffv1'. \
                 Lossy codecs corrupt the colour palette signal — \
                 re-encode with sllv encode or use an FFV1 source.",
                codec
            );
        }
    }
    // ────────────────────────────────────────────────────────────────────────

    std::fs::create_dir_all(out_frames_dir).context("create out frames dir")?;
    let out_pattern = out_frames_dir.join("frame_%06d.png");

    let status = Command::new(&ffmpeg)
        .arg("-y")
        .arg("-i").arg(in_video)
        .arg("-vsync").arg("0")
        .arg("-pix_fmt").arg("rgb24")   // force exact 8-bit-per-channel output
        .arg("-start_number").arg("0")
        .arg(&out_pattern)
        .status()
        .context("spawn ffmpeg (decode)")?;

    if !status.success() {
        bail!("ffmpeg decode failed: {status}");
    }

    Ok(())
}
