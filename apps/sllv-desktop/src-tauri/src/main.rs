#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct Progress {
    done: u64,
    total: u64,
    stage: String,
}

#[tauri::command]
fn encode_path(input: String, out_dir: String) -> Result<()> {
    let input = PathBuf::from(input);
    let out_dir = PathBuf::from(out_dir);

    let (tar_bytes, name) = sllv_core::pack_path_to_tar_bytes(&input)?;
    let p = sllv_core::RasterParams::default();

    sllv_core::encode_bytes_to_frames_dir(&tar_bytes, &format!("{}.tar", name), &out_dir, &p)?;
    Ok(())
}

#[tauri::command]
fn decode_frames(in_dir: String, output: String) -> Result<()> {
    let in_dir = PathBuf::from(in_dir);
    let output = PathBuf::from(output);

    let bytes = sllv_core::decode_frames_dir_to_bytes(&in_dir)?;
    std::fs::write(output, bytes)?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![encode_path, decode_frames])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
