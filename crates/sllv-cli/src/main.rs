mod ffmpeg;
mod interactive;

use anyhow::Context;
use clap::{ArgGroup, Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "sllv",
    version,
    about = "SLLV turns files/folders into TV-static frames and can recover them later.",
    after_help = "Examples:\n  sllv encode -i <path> -o <frames_dir>\n  sllv encode -i <path> -o <frames_dir> --out-mkv out.mkv\n  sllv decode -i <frames_dir> -o recovered/\n  sllv decode -m input.mkv -o recovered/\n  sllv doctor --check-ffmpeg\n\nNotes:\n  - Decode writes the recovered files directly into the output folder.\n  - Encode/decode must use the same --profile (archive vs scan).\n\nTip:\n  - If you double-click sllv.exe on Windows, it opens an interactive menu."
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
        #[arg(long, short = 'i', value_name = "PATH")]
        input: PathBuf,
        #[arg(long, short = 'o', value_name = "DIR")]
        out_frames: PathBuf,
        #[arg(long, value_name = "FILE")]
        out_mkv: Option<PathBuf>,
        #[arg(long, default_value_t = 24)]
        fps: u32,
        #[arg(long, value_enum, default_value_t = ProfileArg::Archive)]
        profile: ProfileArg,
        #[arg(long, value_name = "PATH")]
        ffmpeg_path: Option<PathBuf>,
    },

    /// Decode a frames directory (or an MKV) back into the original files.
    ///
    /// The recovered tar is automatically extracted into the output directory.
    #[command(
        group = ArgGroup::new("source")
            .required(true)
            .args(["input_frames", "input_mkv"])
    )]
    Decode {
        #[arg(long, short = 'i', alias = "input", value_name = "DIR")]
        input_frames: Option<PathBuf>,
        #[arg(long, short = 'm', value_name = "FILE")]
        input_mkv: Option<PathBuf>,
        /// Output directory; recovered files are extracted directly here.
        #[arg(long, short = 'o', value_name = "DIR")]
        out_dir: PathBuf,
        #[arg(long, value_enum, default_value_t = ProfileArg::Archive)]
        profile: ProfileArg,
        #[arg(long, value_name = "PATH")]
        ffmpeg_path: Option<PathBuf>,
    },

    /// Print diagnostic info (and optionally verify ffmpeg is runnable).
    Doctor {
        #[arg(long)]
        check_ffmpeg: bool,
        #[arg(long, value_name = "PATH")]
        ffmpeg_path: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    if std::env::args_os().len() <= 1 {
        return interactive::run();
    }

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
            // --- pre-flight checks -------------------------------------------

            // Friendly error if the input path doesn't exist, before any work
            // starts.  pack_path_to_tar_bytes would also catch this, but its
            // error message is less readable.
            if !input.exists() {
                anyhow::bail!("Input path does not exist: {}", input.display());
            }

            // Refuse to encode into an existing non-empty frames directory.
            // Stale frames from a prior run would mix with the new ones and
            // cause SHA-256 failures on decode with no obvious explanation.
            if out_frames.exists() {
                let is_empty = std::fs::read_dir(&out_frames)
                    .map(|mut d| d.next().is_none())
                    .unwrap_or(false);
                if !is_empty {
                    anyhow::bail!(
                        "Output frames directory already exists and is not empty: {}\n\
                         Remove it or choose a different path to avoid mixing stale frames.",
                        out_frames.display()
                    );
                }
            }

            // --- encode -------------------------------------------------------
            let (tar, name) = sllv_core::pack::pack_path_to_tar_bytes(&input).context("pack input")?;
            let rp = profile.to_profile().defaults();

            let manifest = sllv_core::raster::encode_bytes_to_frames_dir(&tar, &name, &out_frames, &rp)
                .context("encode bytes->frames")?;

            if let Some(out) = out_mkv {
                ffmpeg::frames_to_ffv1_mkv(&out_frames, &out, fps, ffmpeg_path.as_deref())
                    .context("ffmpeg frames->mkv")?;
            }

            println!("Encoded {} frames into {}", manifest.frames, out_frames.display());
        }
        Command::Decode {
            input_frames,
            input_mkv,
            out_dir,
            profile,
            ffmpeg_path,
        } => {
            let (frames_dir, _tmp_guard): (PathBuf, Option<TempDirCleanup>) =
                if let Some(frames) = input_frames {
                    (frames, None)
                } else if let Some(mkv) = input_mkv {
                    let tmp = std::env::temp_dir().join(format!(
                        "sllv_cli_decode_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis()
                    ));
                    ffmpeg::mkv_to_frames(&mkv, &tmp, ffmpeg_path.as_deref()).context("ffmpeg mkv->frames")?;
                    (tmp.clone(), Some(TempDirCleanup { path: tmp }))
                } else {
                    anyhow::bail!("must provide --input-frames or --input-mkv");
                };

            let rp = profile.to_profile().defaults();
            let bytes = sllv_core::raster::decode_frames_dir_to_bytes_with_params(&frames_dir, &rp)
                .context("decode frames")?;

            // Auto-extract the recovered tar into out_dir
            std::fs::create_dir_all(&out_dir).context("create output dir")?;
            let cursor = std::io::Cursor::new(bytes);
            let mut archive = tar::Archive::new(cursor);
            archive.unpack(&out_dir).context("extract tar")?;

            println!("Decoded into {}", out_dir.display());
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
        let ffmpeg_bin = ffmpeg_path.unwrap_or(Path::new("ffmpeg"));
        let ok = std::process::Command::new(ffmpeg_bin)
            .arg("-version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if ok {
            println!("- FFmpeg: ok ({})", ffmpeg_bin.display());
        } else {
            println!("- FFmpeg: NOT found ({})", ffmpeg_bin.display());
            println!("  Install ffmpeg or pass --ffmpeg-path.");
        }
    }

    Ok(())
}

/// RAII guard that removes a temporary directory on drop.
struct TempDirCleanup {
    path: PathBuf,
}

impl Drop for TempDirCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
