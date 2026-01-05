use crate::ffmpeg;
use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "sllv", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(ValueEnum, Clone, Debug)]
enum ProfileArg {
    Archive,
    Scan,
}

impl ProfileArg {
    fn to_profile(&self) -> sllv_core::Profile {
        match self {
            ProfileArg::Archive => sllv_core::Profile::Archive,
            ProfileArg::Scan => sllv_core::Profile::Scan,
        }
    }
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
        #[arg(long, value_enum, default_value_t = ProfileArg::Archive)]
        profile: ProfileArg,
    },
    Decode {
        #[arg(long)]
        input_frames: Option<PathBuf>,
        #[arg(long)]
        input_mkv: Option<PathBuf>,
        #[arg(long)]
        out_tar: PathBuf,
        #[arg(long, value_enum, default_value_t = ProfileArg::Archive)]
        profile: ProfileArg,
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
            profile,
        } => {
            let tar = sllv_core::pack::pack_path_to_tar_bytes(&input).context("pack input")?;
            let (rp, _) = profile.to_profile().defaults();

            let manifest = sllv_core::raster::encode_bytes_to_frames_dir(
                &tar,
                input
                    .file_name()
                    .and_then(|x| x.to_str())
                    .unwrap_or("input"),
                &out_frames,
                &rp,
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
            profile,
        } => {
            let frames_dir = if let Some(frames) = input_frames {
                frames
            } else if let Some(mkv) = input_mkv {
                let tmp = out_tar
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join("_sllv_tmp_frames");
                ffmpeg::mkv_to_frames(&mkv, &tmp).context("ffmpeg mkv->frames")?;
                tmp
            } else {
                anyhow::bail!("must provide --input-frames or --input-mkv");
            };

            let (rp, _) = profile.to_profile().defaults();
            let bytes = sllv_core::raster::decode_frames_dir_to_bytes_with_params(&frames_dir, &rp)
                .context("decode frames")?;
            std::fs::write(&out_tar, bytes).context("write recovered tar")?;
        }
    }

    Ok(())
}
