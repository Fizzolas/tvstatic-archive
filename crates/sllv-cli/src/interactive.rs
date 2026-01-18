use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;

fn prompt_line(label: &str) -> anyhow::Result<String> {
    print!("{label}");
    io::stdout().flush().ok();

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())
}

fn prompt_path(label: &str) -> anyhow::Result<PathBuf> {
    Ok(PathBuf::from(prompt_line(label)?))
}

fn prompt_optional_path(label: &str) -> anyhow::Result<Option<PathBuf>> {
    let s = prompt_line(label)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(PathBuf::from(s)))
    }
}

fn prompt_yes_no(label: &str, default_no: bool) -> anyhow::Result<bool> {
    let hint = if default_no { "[y/N]" } else { "[Y/n]" };
    let s = prompt_line(&format!("{label} {hint}: "))?.to_lowercase();
    if s.is_empty() {
        return Ok(!default_no);
    }
    Ok(matches!(s.as_str(), "y" | "yes"))
}

fn prompt_profile() -> anyhow::Result<sllv_core::Profile> {
    let s = prompt_line("Profile (archive/scan) [archive]: ")?.to_lowercase();
    Ok(match s.as_str() {
        "scan" => sllv_core::Profile::Scan,
        _ => sllv_core::Profile::Archive,
    })
}

fn pause_exit() {
    let _ = prompt_line("\nPress Enter to exit...");
}

pub fn run() -> anyhow::Result<()> {
    println!("SLLV (interactive)\n=================\n");

    loop {
        println!("Choose an action:");
        println!("  1) Encode -> frames");
        println!("  2) Encode -> frames + mkv (ffmpeg)");
        println!("  3) Decode frames -> recovered.tar");
        println!("  4) Decode mkv -> recovered.tar (ffmpeg)");
        println!("  5) Doctor");
        println!("  0) Exit");

        let choice = prompt_line("\nSelection: ")?;
        match choice.trim() {
            "0" => break,
            "1" => {
                let profile = prompt_profile()?;
                let input = prompt_path("Input file/folder path: ")?;
                let out_frames = prompt_path("Output frames directory: ")?;
                let rp = profile.defaults();

                let (tar, name) = sllv_core::pack::pack_path_to_tar_bytes(&input).context("pack input")?;
                let manifest =
                    sllv_core::raster::encode_bytes_to_frames_dir(&tar, &name, &out_frames, &rp).context("encode")?;

                println!("\nOK: Wrote {} frames to {}", manifest.frames, out_frames.display());
            }
            "2" => {
                let profile = prompt_profile()?;
                let input = prompt_path("Input file/folder path: ")?;
                let out_frames = prompt_path("Output frames directory: ")?;
                let out_mkv = prompt_path("Output mkv file path (e.g. out.mkv): ")?;

                let fps_s = prompt_line("FPS [24]: ")?;
                let fps: u32 = fps_s.parse().unwrap_or(24);

                let ffmpeg_path = prompt_optional_path("Optional ffmpeg path (blank = PATH): ")?;

                let rp = profile.defaults();
                let (tar, name) = sllv_core::pack::pack_path_to_tar_bytes(&input).context("pack input")?;
                let manifest =
                    sllv_core::raster::encode_bytes_to_frames_dir(&tar, &name, &out_frames, &rp).context("encode")?;

                crate::ffmpeg::frames_to_ffv1_mkv(&out_frames, &out_mkv, fps, ffmpeg_path.as_deref())
                    .context("ffmpeg frames->mkv")?;

                println!(
                    "\nOK: Wrote {} frames to {} and mkv to {}",
                    manifest.frames,
                    out_frames.display(),
                    out_mkv.display()
                );
            }
            "3" => {
                let profile = prompt_profile()?;
                let input_frames = prompt_path("Input frames directory: ")?;
                let out_tar = prompt_path("Output tar file path (e.g. recovered.tar): ")?;

                let rp = profile.defaults();
                let bytes = sllv_core::raster::decode_frames_dir_to_bytes_with_params(&input_frames, &rp)
                    .context("decode frames")?;
                std::fs::write(&out_tar, bytes).context("write tar")?;

                println!("\nOK: Wrote recovered tar to {}", out_tar.display());
                println!("Tip: extract with: tar -xf \"{}\" -C out_dir", out_tar.display());
            }
            "4" => {
                let profile = prompt_profile()?;
                let input_mkv = prompt_path("Input mkv file path: ")?;
                let out_tar = prompt_path("Output tar file path (e.g. recovered.tar): ")?;
                let ffmpeg_path = prompt_optional_path("Optional ffmpeg path (blank = PATH): ")?;

                let tmp = out_tar
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join("_sllv_tmp_frames");

                crate::ffmpeg::mkv_to_frames(&input_mkv, &tmp, ffmpeg_path.as_deref()).context("ffmpeg mkv->frames")?;

                let rp = profile.defaults();
                let bytes = sllv_core::raster::decode_frames_dir_to_bytes_with_params(&tmp, &rp)
                    .context("decode frames")?;
                std::fs::write(&out_tar, bytes).context("write tar")?;

                println!("\nOK: Wrote recovered tar to {}", out_tar.display());
            }
            "5" => {
                println!("\nSLLV doctor\n----------");
                println!("- Temp dir: {}", std::env::temp_dir().display());

                let tmp = std::env::temp_dir().join("sllv_doctor_write_test.tmp");
                std::fs::write(&tmp, b"ok").context("write temp")?;
                std::fs::remove_file(&tmp).ok();
                println!("- Temp dir write: ok");

                let check_ffmpeg = prompt_yes_no("Check ffmpeg?", true)?;
                if check_ffmpeg {
                    let ffmpeg_path = prompt_optional_path("Optional ffmpeg path (blank = PATH): ")?;
                    let p = ffmpeg_path
                        .as_ref()
                        .map(|x| x.display().to_string())
                        .unwrap_or_else(|| "(PATH)".to_string());

                    match crate::ffmpeg::mkv_to_frames(Path::new("__nonexistent__.mkv"), Path::new("."), ffmpeg_path.as_deref()) {
                        Ok(_) => println!("- FFmpeg: ok ({p})"),
                        Err(e) => {
                            let msg = format!("{e:#}").to_lowercase();
                            if msg.contains("ffmpeg not found") || msg.contains("not found") {
                                println!("- FFmpeg: missing ({p})");
                                println!("  Install ffmpeg or provide a full path.");
                            } else {
                                println!("- FFmpeg: ok ({p})");
                            }
                        }
                    }
                }
            }
            _ => {
                println!("\nUnknown selection: {choice}");
            }
        }

        // Keep things readable in a double-clicked console.
        let _ = prompt_line("\nPress Enter to return to menu...");
        println!();
    }

    pause_exit();
    Ok(())
}
