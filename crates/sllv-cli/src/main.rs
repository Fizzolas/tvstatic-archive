use crate::ffmpeg;
use anyhow::Context;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sllv", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    Encode {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        out_frames: PathBuf,
        #[arg(long)]
        out_mkv: Option<PathBuf>,
        #[arg(long, default_value_t = 24)]
        fps: u32,
    },
    Decode {
        #[arg(long)]
        input_frames: Option<PathBuf>,
        #[arg(long)]
        input_mkv: Option<PathBuf>,
        #[arg(long)]
        out_tar: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Encode {
            input,
            out_frames,
            out_mkv,
            fps,
        } => {
            let tar = sllv_core::pack::pack_path_to_tar_bytes(&input).context("pack input")?;
            let manifest = sllv_core::raster::encode_bytes_to_frames_dir(
                &tar,
                input
                    .file_name()
                    .and_then(|x| x.to_str())
                    .unwrap_or("input"),
                &out_frames,
                &sllv_core::raster::RasterParams::default(),
            )
            .context("encode bytes->frames")?;

            if let Some(out) = out_mkv {
                ffmpeg::frames_to_ffv1_mkv(&out_frames, &out, fps).context("ffmpeg frames->mkv")?;
            }

            println!("Frames: {}", manifest.frames);
        }
        Command::Decode {
            input_frames,
            input_mkv,
            out_tar,
        } => {
            let frames_dir = if let Some(frames) = input_frames {
                frames
            } else if let Some(mkv) = input_mkv {
                // Extract into a sibling folder.
                let tmp = out_tar
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("_sllv_tmp_frames");
                ffmpeg::mkv_to_frames(&mkv, &tmp).context("ffmpeg mkv->frames")?;
                tmp
            } else {
                anyhow::bail!("must provide --input-frames or --input-mkv");
            };

            let bytes = sllv_core::raster::decode_frames_dir_to_bytes(&frames_dir)
                .context("decode frames")?;
            std::fs::write(&out_tar, bytes).context("write recovered tar")?;
        }
    }

    Ok(())
}
