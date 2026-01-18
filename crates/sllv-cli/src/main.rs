mod ffmpeg;

use anyhow::Context;
use clap::{ArgGroup, Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "sllv",
    version,
    about = "SLLV turns files/folders into TV-static frames and can recover them later.",
    after_help = "Examples:\n  sllv encode -i <path> -o <frames_dir>\n  sllv encode -i <path> -o <frames_dir> --out-mkv out.mkv\n  sllv decode -i <frames_dir> -o recovered.tar\n  sllv decode -m input.mkv -o recovered.tar\n  sllv doctor --check-ffmpeg\n\nNotes:\n  - Decode always outputs a .tar file; extract it with: tar -xf recovered.tar -C out_dir\n  - Encode/decode must use the same --profile (archive vs scan)."
)]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(ValueEnum, Clone, Debug)]
enum ProfileArg {
    /// Lossless / exact pixel path (best for storing as PNG frames or truly lossless video).
    Archive,
    /// Robust path intended for camera/screen pipelines (deskew + FEC).
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
    /// Encode a file or folder into a directory of PNG frames (and optionally an MKV via ffmpeg).
    Encode {
        /// File/folder to encode (it will be packed into a tar internally).
        #[arg(long, short = 'i', value_name = "PATH")]
        input: PathBuf,

        /// Output directory that will receive frame_000000.png, manifest.json, etc.
        #[arg(long, short = 'o', value_name = "DIR")]
        out_frames: PathBuf,

        /// Optional output MKV path (FFV1 in Matroska). Requires ffmpeg.
        #[arg(long, value_name = "FILE")]
        out_mkv: Option<PathBuf>,

        /// FPS to use when writing an MKV (ignored unless --out-mkv is set).
        #[arg(long, default_value_t = 24)]
        fps: u32,

        /// Preset controlling encoding/decoding parameters.
        #[arg(long, value_enum, default_value_t = ProfileArg::Archive)]
        profile: ProfileArg,

        /// Optional path to an ffmpeg executable (avoids needing it on PATH).
        #[arg(long, value_name = "PATH")]
        ffmpeg_path: Option<PathBuf>,
    },

    /// Decode a frames directory (or an MKV) back into a .tar archive.
    #[command(
        group = ArgGroup::new("source")
            .required(true)
            .args(["input_frames", "input_mkv"])
    )]
    Decode {
        /// Directory containing frames + manifest.json.
        #[arg(long, short = 'i', alias = "input", value_name = "DIR")]
        input_frames: Option<PathBuf>,

        /// Input MKV path; frames will be extracted to a temp dir first.
        #[arg(long, short = 'm', value_name = "FILE")]
        input_mkv: Option<PathBuf>,

        /// Output tar file path (the recovered data is written here).
        #[arg(long, short = 'o', value_name = "FILE")]
        out_tar: PathBuf,

        /// Preset controlling decoding parameters; must match what was used for encode.
        #[arg(long, value_enum, default_value_t = ProfileArg::Archive)]
        profile: ProfileArg,

        /// Optional path to an ffmpeg executable.
        #[arg(long, value_name = "PATH")]
        ffmpeg_path: Option<PathBuf>,
    },

    /// Print diagnostic info (and optionally verify ffmpeg is runnable).
    Doctor {
        /// Also check ffmpeg availability.
        #[arg(long)]
        check_ffmpeg: bool,

        /// Optional path to an ffmpeg executable.
        #[arg(long, value_name = "PATH")]
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
            let (tar, name) = sllv_core::pack::pack_path_to_tar_bytes(&input).context("pack input")?;
            let rp = profile.to_profile().defaults();

            let manifest = sllv_core::raster::encode_bytes_to_frames_dir(&tar, &name, &out_frames, &rp)
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
                ffmpeg::mkv_to_frames(&mkv, &tmp, ffmpeg_path.as_deref()).context("ffmpeg mkv->frames")?;
                tmp
            } else {
                anyhow::bail!("must provide --input-frames or --input-mkv");
            };

            let rp = profile.to_profile().defaults();
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

    println!("- Temp dir: {}", std::env::temp_dir().display());

    let tmp = std::env::temp_dir().join("sllv_doctor_write_test.tmp");
    std::fs::write(&tmp, b"ok").context("write temp")?;
    std::fs::remove_file(&tmp).ok();
    println!("- Temp dir write: ok");

    if check_ffmpeg {
        let p = ffmpeg_path
            .map(|x| x.display().to_string())
            .unwrap_or_else(|| "(PATH)".to_string());

        // This call will succeed only if ffmpeg is runnable; it may still error due to missing input.
        match ffmpeg::mkv_to_frames(Path::new("__nonexistent__.mkv"), Path::new("."), ffmpeg_path) {
            Ok(_) => println!("- FFmpeg: ok ({p})"),
            Err(e) => {
                let msg = format!("{e:#}").to_lowercase();
                if msg.contains("ffmpeg not found") || msg.contains("not found") {
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
