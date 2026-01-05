#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

#[derive(Default)]
struct Busy(std::sync::Mutex<bool>);

#[derive(Serialize, Clone)]
struct Progress {
    stage: String,
    done: u64,
    total: u64,
}

#[derive(Serialize, Clone)]
struct TaskResult {
    ok: bool,
    message: String,
}

#[tauri::command]
fn encode_path(app: AppHandle, busy: State<'_, Busy>, input: String, out_dir: String) -> Result<()> {
    {
        let mut b = busy.0.lock().unwrap();
        if *b {
            anyhow::bail!("busy");
        }
        *b = true;
    }

    std::thread::spawn(move || {
        let res: anyhow::Result<()> = (|| {
            app.emit("progress", Progress { stage: "Packing input".into(), done: 0, total: 1 })?;
            let input = PathBuf::from(input);
            let out_dir = PathBuf::from(out_dir);
            let (tar_bytes, name) = sllv_core::pack_path_to_tar_bytes(&input)?;

            app.emit(
                "progress",
                Progress {
                    stage: "Encoding frames".into(),
                    done: 0,
                    total: tar_bytes.len() as u64,
                },
            )?;

            let p = sllv_core::RasterParams::default();
            let _m = sllv_core::encode_bytes_to_frames_dir(&tar_bytes, &format!("{}.tar", name), &out_dir, &p)?;

            app.emit(
                "progress",
                Progress {
                    stage: "Done".into(),
                    done: tar_bytes.len() as u64,
                    total: tar_bytes.len() as u64,
                },
            )?;
            Ok(())
        })();

        let msg = match res {
            Ok(_) => TaskResult { ok: true, message: "Encode complete".into() },
            Err(e) => TaskResult { ok: false, message: format!("Encode failed: {e}") },
        };
        let _ = app.emit("task_result", msg);
        // clear busy
        if let Some(busy) = app.try_state::<Busy>() {
            let mut b = busy.0.lock().unwrap();
            *b = false;
        }
    });

    Ok(())
}

#[tauri::command]
fn decode_frames(app: AppHandle, busy: State<'_, Busy>, in_dir: String, output: String) -> Result<()> {
    {
        let mut b = busy.0.lock().unwrap();
        if *b {
            anyhow::bail!("busy");
        }
        *b = true;
    }

    std::thread::spawn(move || {
        let res: anyhow::Result<()> = (|| {
            app.emit("progress", Progress { stage: "Decoding frames".into(), done: 0, total: 1 })?;
            let in_dir = PathBuf::from(in_dir);
            let output = PathBuf::from(output);
            let bytes = sllv_core::decode_frames_dir_to_bytes(&in_dir)?;
            app.emit(
                "progress",
                Progress { stage: "Writing output".into(), done: 0, total: bytes.len() as u64 },
            )?;
            std::fs::write(output, bytes)?;
            app.emit(
                "progress",
                Progress { stage: "Done".into(), done: 1, total: 1 },
            )?;
            Ok(())
        })();

        let msg = match res {
            Ok(_) => TaskResult { ok: true, message: "Decode complete".into() },
            Err(e) => TaskResult { ok: false, message: format!("Decode failed: {e}") },
        };
        let _ = app.emit("task_result", msg);
        if let Some(busy) = app.try_state::<Busy>() {
            let mut b = busy.0.lock().unwrap();
            *b = false;
        }
    });

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(Busy::default())
        .invoke_handler(tauri::generate_handler![encode_path, decode_frames])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
