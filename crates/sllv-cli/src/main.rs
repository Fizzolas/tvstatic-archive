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
        input_frames: PathBuf,
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
            let manifest = sllv_core::pack::pack_path_to_tar_bytes(&input)
                .context("pack input into tar bytes")
                .and_then(|tar| {
                    sllv_core::raster::encode_bytes_to_frames_dir(
                        &tar,
                        input
                            .file_name()
                            .and_then(|x| x.to_str())
                            .unwrap_or("input"),
                        &out_frames,
                        &sllv_core::raster::RasterParams::default(),
                    )
                    .map_err(anyhow::Error::from)
                })?;

            if let Some(out) = out_mkv {
                ffmpeg::frames_to_ffv1_mkv(&out_frames, &out, fps).context("ffmpeg frames->mkv")?;
            }

            println!("Frames: {}", manifest.frames);
        }
        Command::Decode { input_frames, out_tar } => {
            let bytes = sllv_core::raster::decode_frames_dir_to_bytes(&input_frames)
                .context("decode frames dir")?;
            std::fs::write(out_tar, bytes).context("write recovered tar")?;
        }
    }

    Ok(())
}
