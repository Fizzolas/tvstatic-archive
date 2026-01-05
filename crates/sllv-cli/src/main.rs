use crate::ffmpeg;
use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};

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
        /// Optional path to an ffmpeg executable (avoids needing it on PATH).
        #[arg(long)]
        ffmpeg_path: Option<PathBuf>,
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
        /// Optional path to an ffmpeg executable (avoids needing it on PATH).
        #[arg(long)]
        ffmpeg_path: Option<PathBuf>,
    },
    Doctor {
        /// Also check ffmpeg availability.
        #[arg(long)]
        check_ffmpeg: bool,
        /// Optional path to an ffmpeg executable.
        #[arg(long)]
        ffmpeg_path: Option<PathBuf>,
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
            ffmpeg_path,
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
                ffmpeg::frames_to_ffv1_mkv(&out_frames, &out, fps, ffmpeg_path.as_deref())
                    .context("ffmpeg frames->mkv")?;
            }

            println!("Frames: {}", manifest.frames);
        }
        Command::Decode {
            input_frames,
            input_mkv,
            out_tar,
            profile,
            ffmpeg_path,
        } => {
            let frames_dir = if let Some(frames) = input_frames {
                frames
            } else if let Some(mkv) = input_mkv {
                let tmp = out_tar
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join("_sllv_tmp_frames");
                ffmpeg::mkv_to_frames(&mkv, &tmp, ffmpeg_path.as_deref())
                    .context("ffmpeg mkv->frames")?;
                tmp
            } else {
                anyhow::bail!("must provide --input-frames or --input-mkv");
            };

            let (rp, _) = profile.to_profile().defaults();
            let bytes = sllv_core::raster::decode_frames_dir_to_bytes_with_params(&frames_dir, &rp)
                .context("decode frames")?;
            std::fs::write(&out_tar, bytes).context("write recovered tar")?;
        }
        Command::Doctor {
            check_ffmpeg,
            ffmpeg_path,
        } => {
            run_doctor(check_ffmpeg, ffmpeg_path.as_deref())?;
        }
    }

    Ok(())
}

fn run_doctor(check_ffmpeg: bool, ffmpeg_path: Option<&Path>) -> anyhow::Result<()> {
    println!("SLLV doctor");

    // Basic info
    println!("- Rust: ok (running binary)");

    // Temp write test
    let tmp = std::env::temp_dir().join("sllv_doctor_write_test.tmp");
    std::fs::write(&tmp, b"ok").context("write temp")?;
    std::fs::remove_file(&tmp).ok();
    println!("- Temp dir write: ok ({})", std::env::temp_dir().display());

    if check_ffmpeg {
        // Delegate to ffmpeg module resolve/probe via a harmless mkv_to_frames argument check.
        // We just call the internal probe via mkv_to_frames with a non-existent file? No: keep simple.
        let p = ffmpeg_path
            .map(|x| x.display().to_string())
            .unwrap_or_else(|| "(PATH)".to_string());
        match crate::ffmpeg::mkv_to_frames(Path::new("__nonexistent__.mkv"), Path::new("."), ffmpeg_path) {
            Ok(_) => {
                println!("- FFmpeg: ok ({p})");
            }
            Err(e) => {
                // If ffmpeg is available, this will usually fail because input doesn't exist.
                // So distinguish 'not found' from 'input missing' loosely.
                let msg = format!("{e:#}");
                if msg.to_lowercase().contains("not found") {
                    println!("- FFmpeg: missing ({p})");
                    println!("  Install ffmpeg or pass --ffmpeg-path.");
                } else {
                    println!("- FFmpeg: ok ({p})");
                }
            }
        }
    }

    Ok(())
}
