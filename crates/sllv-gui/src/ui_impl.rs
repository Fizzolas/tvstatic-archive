use crate::state::AppState;
use crate::ui::{help_button, HelpTopic, Tab};
use eframe::egui;
use std::sync::mpsc;
use std::thread;

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll progress channel
        if let Some(rx) = &self.progress_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    sllv_core::raster::ProgressMsg::Stage { name, done, total } => {
                        if let Some(ref mut prog) = self.progress {
                            prog.stage = name;
                            prog.done = done;
                            prog.total = total;
                        } else {
                            self.progress = Some(crate::state::Progress {
                                stage: name,
                                done,
                                total,
                                started_at: std::time::Instant::now(),
                            });
                        }
                        ctx.request_repaint();
                    }
                    sllv_core::raster::ProgressMsg::Done => {
                        self.is_running = false;
                        self.progress = None;
                        self.progress_rx = None;
                        ctx.request_repaint();
                    }
                    sllv_core::raster::ProgressMsg::Error(e) => {
                        self.log.push_str(&format!("Error: {e}\n"));
                        self.is_running = false;
                        self.progress = None;
                        self.progress_rx = None;
                        ctx.request_repaint();
                    }
                }
            }
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Encode, "Encode");
                ui.selectable_value(&mut self.tab, Tab::Decode, "Decode");
                ui.selectable_value(&mut self.tab, Tab::Doctor, "Doctor");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Encode => ui_encode(ui, self),
            Tab::Decode => ui_decode(ui, self),
            Tab::Doctor => ui_doctor(ui, self),
        });

        egui::TopBottomPanel::bottom("log").resizable(true).show(ctx, |ui| {
            if let Some(ref prog) = self.progress {
                ui.label(format!("Stage: {} ({}/{})", prog.stage, prog.done, prog.total));
                let frac = if prog.total > 0 {
                    (prog.done as f32) / (prog.total as f32)
                } else {
                    0.0
                };
                ui.add(egui::ProgressBar::new(frac).show_percentage());
                if let Some(eta) = prog.eta_secs() {
                    ui.label(format!("ETA: {}s", eta));
                }
                ui.separator();
            }

            ui.label("Log");
            egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut self.log).desired_rows(6));
            });
            if ui.button("Copy log").clicked() {
                ui.output_mut(|o| o.copied_text = self.log.clone());
            }
        });

        if let Some(topic) = self.show_help {
            egui::Window::new(topic.title())
                .collapsible(false)
                .resizable(true)
                .open(&mut true)
                .show(ctx, |ui| {
                    ui.label(topic.body());
                    if ui.button("Close").clicked() {
                        self.show_help = None;
                    }
                });
        }
    }
}

fn ui_encode(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Encode");

    ui.horizontal(|ui| {
        ui.label("Profile");
        help_button(ui, state, HelpTopic::Profile);
        let mut p = match state.encode.profile {
            sllv_core::Profile::Archive => 0,
            sllv_core::Profile::Scan => 1,
        };
        egui::ComboBox::from_id_source("encode_profile")
            .selected_text(if p == 0 { "Archive" } else { "Scan" })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut p, 0, "Archive");
                ui.selectable_value(&mut p, 1, "Scan");
            });
        let new_profile = if p == 0 {
            sllv_core::Profile::Archive
        } else {
            sllv_core::Profile::Scan
        };
        if new_profile.name() != state.encode.profile.name() {
            state.encode.profile = new_profile;
            state.encode.rp = new_profile.defaults();
        }
    });

    ui.separator();

    ui.label(format!(
        "Input: {}",
        state
            .encode
            .input
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(not set)".into())
    ));
    ui.horizontal(|ui| {
        if ui.button("Choose input file").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                state.encode.input = Some(path);
            }
        }
        if ui.button("Choose input folder").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                state.encode.input = Some(path);
            }
        }
    });

    ui.label(format!(
        "Output frames dir: {}",
        state
            .encode
            .out_frames
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(not set)".into())
    ));
    if ui.button("Choose output frames folder").clicked() {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            state.encode.out_frames = Some(path);
        }
    }

    ui.separator();

    ui.collapsing("Safe settings (keep consistent for decode)", |ui| {
        ui.horizontal(|ui| {
            ui.label("Cell size (px)");
            help_button(ui, state, HelpTopic::CellPx);
            ui.add(egui::DragValue::new(&mut state.encode.rp.cell_px).clamp_range(1..=32));
        });
        ui.horizontal(|ui| {
            ui.label("Border cells");
            help_button(ui, state, HelpTopic::BorderCells);
            ui.add(egui::DragValue::new(&mut state.encode.rp.border_cells).clamp_range(0..=64));
        });
        ui.horizontal(|ui| {
            ui.label("Fiducial size (cells)");
            help_button(ui, state, HelpTopic::FiducialSize);
            ui.add(egui::DragValue::new(&mut state.encode.rp.fiducial_size_cells).clamp_range(4..=64));
        });
        ui.horizontal(|ui| {
            ui.label("Deskew");
            help_button(ui, state, HelpTopic::Deskew);
            ui.checkbox(&mut state.encode.rp.deskew, "Enable");
        });

        let show_fec = state.encode.rp.fec.is_some();
        if show_fec {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Error correction (FEC)");
                help_button(ui, state, HelpTopic::Fec);
            });

            if let Some(ref mut fec) = state.encode.rp.fec {
                ui.horizontal(|ui| {
                    ui.label("Data shards");
                    ui.add(egui::DragValue::new(&mut fec.data_shards).clamp_range(1..=64));
                });
                ui.horizontal(|ui| {
                    ui.label("Parity shards");
                    ui.add(egui::DragValue::new(&mut fec.parity_shards).clamp_range(0..=64));
                });
                ui.horizontal(|ui| {
                    ui.label("Shard bytes");
                    ui.add(egui::DragValue::new(&mut fec.shard_bytes).clamp_range(64..=4096));
                });
            }
        }
    });

    ui.separator();

    ui.collapsing("MKV / FFmpeg (optional)", |ui| {
        ui.label(format!(
            "Output MKV: {}",
            state
                .encode
                .out_mkv
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(disabled)".into())
        ));
        ui.horizontal(|ui| {
            if ui.button("Choose MKV output").clicked() {
                state.encode.out_mkv = rfd::FileDialog::new().add_filter("Matroska", &["mkv"]).save_file();
            }
            if ui.button("Disable MKV").clicked() {
                state.encode.out_mkv = None;
            }
        });

        ui.horizontal(|ui| {
            ui.label("FPS");
            help_button(ui, state, HelpTopic::Fps);
            ui.add(egui::DragValue::new(&mut state.encode.fps).clamp_range(1..=240));
        });

        ui.horizontal(|ui| {
            ui.label("FFmpeg path");
            help_button(ui, state, HelpTopic::Ffmpeg);
            ui.label(
                state
                    .encode
                    .ffmpeg_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "(PATH)".into()),
            );
            if ui.button("Choose ffmpeg.exe").clicked() {
                state.encode.ffmpeg_path = rfd::FileDialog::new().add_filter("ffmpeg", &["exe"]).pick_file();
            }
            if ui.button("Clear").clicked() {
                state.encode.ffmpeg_path = None;
            }
        });
    });

    ui.separator();

    ui.add_enabled_ui(!state.is_running, |ui| {
        if ui.button("Start encode").clicked() {
            spawn_encode_thread(state);
        }
    });
}

fn ui_decode(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Decode");

    ui.horizontal(|ui| {
        ui.label("Profile");
        help_button(ui, state, HelpTopic::Profile);
        let mut p = match state.decode.profile {
            sllv_core::Profile::Archive => 0,
            sllv_core::Profile::Scan => 1,
        };
        egui::ComboBox::from_id_source("decode_profile")
            .selected_text(if p == 0 { "Archive" } else { "Scan" })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut p, 0, "Archive");
                ui.selectable_value(&mut p, 1, "Scan");
            });
        let new_profile = if p == 0 {
            sllv_core::Profile::Archive
        } else {
            sllv_core::Profile::Scan
        };
        if new_profile.name() != state.decode.profile.name() {
            state.decode.profile = new_profile;
            state.decode.rp = new_profile.defaults();
        }
    });

    ui.separator();

    ui.collapsing("Input (choose one)", |ui| {
        ui.label(format!(
            "Frames folder: {}",
            state
                .decode
                .input_frames
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(not set)".into())
        ));
        if ui.button("Choose frames folder").clicked() {
            state.decode.input_frames = rfd::FileDialog::new().pick_folder();
        }

        ui.label(format!(
            "MKV file: {}",
            state
                .decode
                .input_mkv
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(not set)".into())
        ));
        if ui.button("Choose MKV file").clicked() {
            state.decode.input_mkv = rfd::FileDialog::new().add_filter("Matroska", &["mkv"]).pick_file();
        }

        if ui.button("Use frames only").clicked() {
            state.decode.input_mkv = None;
        }
        if ui.button("Use mkv only").clicked() {
            state.decode.input_frames = None;
        }
    });

    ui.separator();

    ui.label(format!(
        "Output tar: {}",
        state
            .decode
            .out_tar
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(not set)".into())
    ));
    if ui.button("Choose output .tar").clicked() {
        state.decode.out_tar = rfd::FileDialog::new().add_filter("tar", &["tar"]).save_file();
    }

    ui.separator();

    ui.collapsing("Safe settings (keep consistent for decode)", |ui| {
        ui.horizontal(|ui| {
            ui.label("Cell size (px)");
            help_button(ui, state, HelpTopic::CellPx);
            ui.add(egui::DragValue::new(&mut state.decode.rp.cell_px).clamp_range(1..=32));
        });
        ui.horizontal(|ui| {
            ui.label("Border cells");
            help_button(ui, state, HelpTopic::BorderCells);
            ui.add(egui::DragValue::new(&mut state.decode.rp.border_cells).clamp_range(0..=64));
        });
        ui.horizontal(|ui| {
            ui.label("Fiducial size (cells)");
            help_button(ui, state, HelpTopic::FiducialSize);
            ui.add(egui::DragValue::new(&mut state.decode.rp.fiducial_size_cells).clamp_range(4..=64));
        });
        ui.horizontal(|ui| {
            ui.label("Deskew");
            help_button(ui, state, HelpTopic::Deskew);
            ui.checkbox(&mut state.decode.rp.deskew, "Enable");
        });

        let show_fec = state.decode.rp.fec.is_some();
        if show_fec {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Error correction (FEC)");
                help_button(ui, state, HelpTopic::Fec);
            });

            if let Some(ref mut fec) = state.decode.rp.fec {
                ui.horizontal(|ui| {
                    ui.label("Data shards");
                    ui.add(egui::DragValue::new(&mut fec.data_shards).clamp_range(1..=64));
                });
                ui.horizontal(|ui| {
                    ui.label("Parity shards");
                    ui.add(egui::DragValue::new(&mut fec.parity_shards).clamp_range(0..=64));
                });
                ui.horizontal(|ui| {
                    ui.label("Shard bytes");
                    ui.add(egui::DragValue::new(&mut fec.shard_bytes).clamp_range(64..=4096));
                });
            }
        }
    });

    ui.separator();

    ui.collapsing("FFmpeg (only if using MKV)", |ui| {
        ui.horizontal(|ui| {
            ui.label("FFmpeg path");
            help_button(ui, state, HelpTopic::Ffmpeg);
            ui.label(
                state
                    .decode
                    .ffmpeg_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "(PATH)".into()),
            );
            if ui.button("Choose ffmpeg.exe").clicked() {
                state.decode.ffmpeg_path = rfd::FileDialog::new().add_filter("ffmpeg", &["exe"]).pick_file();
            }
            if ui.button("Clear").clicked() {
                state.decode.ffmpeg_path = None;
            }
        });
    });

    ui.separator();

    ui.add_enabled_ui(!state.is_running, |ui| {
        if ui.button("Start decode").clicked() {
            spawn_decode_thread(state);
        }
    });
}

fn ui_doctor(ui: &mut egui::Ui, state: &mut AppState) {
    ui.heading("Doctor");
    ui.label("Checks basic things that commonly break installs.");

    if ui.button("Run doctor").clicked() {
        match run_doctor() {
            Ok(msg) => state.log.push_str(&format!("{msg}\n")),
            Err(e) => state.log.push_str(&format!("Doctor failed: {e:#}\n")),
        }
    }
}

fn run_doctor() -> anyhow::Result<String> {
    let tmp = std::env::temp_dir().join("sllv_doctor_write_test.tmp");
    std::fs::write(&tmp, b"ok")?;
    std::fs::remove_file(&tmp).ok();
    Ok(format!("Doctor OK. Temp dir: {}", std::env::temp_dir().display()))
}

fn spawn_encode_thread(state: &mut AppState) {
    let input = match state.encode.input.as_ref() {
        Some(p) => p.clone(),
        None => {
            state.log.push_str("Error: Input not set\n");
            return;
        }
    };
    let out_frames = match state.encode.out_frames.as_ref() {
        Some(p) => p.clone(),
        None => {
            state.log.push_str("Error: Output frames folder not set\n");
            return;
        }
    };

    let out_mkv = state.encode.out_mkv.clone();
    let fps = state.encode.fps;
    let ffmpeg_path = state.encode.ffmpeg_path.clone();
    let rp = state.encode.rp.clone();

    let (tx, rx) = mpsc::channel();
    state.progress_rx = Some(rx);
    state.is_running = true;
    state.progress = Some(crate::state::Progress {
        stage: "starting".into(),
        done: 0,
        total: 1,
        started_at: std::time::Instant::now(),
    });

    thread::spawn(move || {
        let res = (|| -> anyhow::Result<()> {
            let (tar, name) = sllv_core::pack::pack_path_to_tar_bytes(&input)?;
            sllv_core::raster::encode_bytes_to_frames_dir_with_progress(&tar, &name, &out_frames, &rp, Some(tx.clone()))?;

            if let Some(out) = out_mkv {
                sllv_core::ffmpeg::frames_to_ffv1_mkv(&out_frames, &out, fps, ffmpeg_path.as_deref())?;
            }
            Ok(())
        })();

        match res {
            Ok(()) => {
                let _ = tx.send(sllv_core::raster::ProgressMsg::Done);
            }
            Err(e) => {
                let _ = tx.send(sllv_core::raster::ProgressMsg::Error(format!("{e:#}")));
            }
        }
    });
}

fn spawn_decode_thread(state: &mut AppState) {
    let out_tar = match state.decode.out_tar.as_ref() {
        Some(p) => p.clone(),
        None => {
            state.log.push_str("Error: Output .tar not set\n");
            return;
        }
    };

    let input_mkv = state.decode.input_mkv.clone();
    let input_frames = state.decode.input_frames.clone();
    let ffmpeg_path = state.decode.ffmpeg_path.clone();
    let rp = state.decode.rp.clone();

    let (tx, rx) = mpsc::channel();
    state.progress_rx = Some(rx);
    state.is_running = true;
    state.progress = Some(crate::state::Progress {
        stage: "starting".into(),
        done: 0,
        total: 1,
        started_at: std::time::Instant::now(),
    });

    thread::spawn(move || {
        let res = (|| -> anyhow::Result<()> {
            let frames_dir: std::path::PathBuf;
            let _temp_guard;

            if let Some(mkv) = input_mkv {
                let tmp = std::env::temp_dir().join(format!(
                    "sllv_gui_decode_frames_{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                ));
                std::fs::create_dir_all(&tmp)?;
                sllv_core::ffmpeg::mkv_to_frames(&mkv, &tmp, ffmpeg_path.as_deref())?;
                frames_dir = tmp;
                _temp_guard = TempDirCleanup { path: frames_dir.clone() };
            } else if let Some(frames) = input_frames {
                frames_dir = frames;
                _temp_guard = TempDirCleanup { path: std::path::PathBuf::new() };
            } else {
                anyhow::bail!("Choose a frames folder or an MKV file");
            }

            let bytes = sllv_core::raster::decode_frames_dir_to_bytes_with_progress(&frames_dir, &rp, Some(tx.clone()))?;
            std::fs::write(&out_tar, bytes)?;
            Ok(())
        })();

        match res {
            Ok(()) => {
                let _ = tx.send(sllv_core::raster::ProgressMsg::Done);
            }
            Err(e) => {
                let _ = tx.send(sllv_core::raster::ProgressMsg::Error(format!("{e:#}")));
            }
        }
    });
}

struct TempDirCleanup {
    path: std::path::PathBuf,
}

impl Drop for TempDirCleanup {
    fn drop(&mut self) {
        if self.path.as_os_str().is_empty() {
            return;
        }
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
