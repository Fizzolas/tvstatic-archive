use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "sllv", version, about = "Static-Lattice Lossless Video tools")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Encode any input path (file or folder) into numbered PNG frames.
    Encode {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long, default_value_t = 256)]
        grid_w: u32,
        #[arg(long, default_value_t = 256)]
        grid_h: u32,
        #[arg(long, default_value_t = 2)]
        cell_px: u32,
        #[arg(long, default_value_t = 24 * 1024)]
        chunk_bytes: u32,
    },

    /// Decode numbered PNG frames back into the original tar archive.
    Decode {
        #[arg(long)]
        in_dir: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },

    /// Create a lossless MKV (FFV1) from frames using ffmpeg.
    MakeVideo {
        #[arg(long)]
        in_dir: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value_t = 30)]
        fps: u32,
    },

    /// Extract frames from a video using ffmpeg (for round-trip testing).
    ExtractFrames {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long, default_value_t = 30)]
        fps: u32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Encode {
            input,
            out_dir,
            grid_w,
            grid_h,
            cell_px,
            chunk_bytes,
        } => {
            let (tar_bytes, name) = sllv_core::pack_path_to_tar_bytes(&input)
                .with_context(|| format!("pack input {:?} to tar", input))?;

            let p = sllv_core::RasterParams {
                grid_w,
                grid_h,
                cell_px,
                chunk_bytes,
                palette: sllv_core::Palette8::Basic,
            };

            let m = sllv_core::encode_bytes_to_frames_dir(&tar_bytes, &format!("{}.tar", name), &out_dir, &p)
                .with_context(|| "encode frames")?;

            println!("Packed as {} ({} bytes)", m.file_name, m.total_bytes);
            println!("Wrote {} frames to {:?}", m.frames, out_dir);
        }

        Cmd::Decode { in_dir, output } => {
            let bytes = sllv_core::decode_frames_dir_to_bytes(&in_dir).with_context(|| "decode")?;
            fs::write(&output, bytes).with_context(|| format!("write {:?}", output))?;
            println!("Wrote recovered archive to {:?}", output);
        }

        Cmd::MakeVideo { in_dir, output, fps } => {
            ensure_ffmpeg()?;
            let input_glob = in_dir.join("frame_%06d.png");
            let status = Command::new("ffmpeg")
                .args([
                    "-y",
                    "-framerate",
                    &fps.to_string(),
                    "-i",
                    input_glob.to_string_lossy().as_ref(),
                    "-c:v",
                    "ffv1",
                    "-pix_fmt",
                    "rgb24",
                    output.to_string_lossy().as_ref(),
                ])
                .status()
                .with_context(|| "run ffmpeg")?;
            if !status.success() {
                bail!("ffmpeg failed");
            }
            println!("Wrote lossless video to {:?}", output);
        }

        Cmd::ExtractFrames { input, out_dir, fps } => {
            ensure_ffmpeg()?;
            fs::create_dir_all(&out_dir)?;
            let out_pattern = out_dir.join("frame_%06d.png");
            let status = Command::new("ffmpeg")
                .args([
                    "-y",
                    "-i",
                    input.to_string_lossy().as_ref(),
                    "-vf",
                    &format!("fps={}", fps),
                    out_pattern.to_string_lossy().as_ref(),
                ])
                .status()
                .with_context(|| "run ffmpeg")?;
            if !status.success() {
                bail!("ffmpeg failed");
            }
            println!("Extracted frames to {:?}", out_dir);
        }
    }

    Ok(())
}

fn ensure_ffmpeg() -> Result<()> {
    let out = Command::new("ffmpeg").arg("-version").output();
    match out {
        Ok(o) if o.status.success() => Ok(()),
        _ => bail!("ffmpeg not found on PATH"),
    }
}
