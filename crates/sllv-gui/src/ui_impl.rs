use crate::state::AppState;
use crate::ui::{help_button, HelpTopic, Tab};
use eframe::egui;

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        // Fix E0499: render FEC header (needs &mut state for help_button) without holding `ref mut fec`.
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

    if ui.button("Start encode").clicked() {
        match run_encode(state) {
            Ok(()) => state.log.push_str("Encode OK\n"),
            Err(e) => state.log.push_str(&format!("Encode failed: {e:#}\n")),
        }
    }
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

    if ui.button("Start decode").clicked() {
        match run_decode(state) {
            Ok(()) => state.log.push_str("Decode OK\n"),
            Err(e) => state.log.push_str(&format!("Decode failed: {e:#}\n")),
        }
    }
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

fn run_encode(state: &mut AppState) -> anyhow::Result<()> {
    let input = state.encode.input.as_ref().ok_or_else(|| anyhow::anyhow!("Input not set"))?;
    let out_frames = state.encode.out_frames.as_ref().ok_or_else(|| anyhow::anyhow!("Output frames folder not set"))?;

    let (tar, name) = sllv_core::pack::pack_path_to_tar_bytes(input)?;
    sllv_core::raster::encode_bytes_to_frames_dir(&tar, &name, out_frames, &state.encode.rp)?;

    if state.encode.out_mkv.is_some() {
        return Err(anyhow::anyhow!(
            "MKV output not wired in GUI yet. Use CLI for MKV or wait for next patch."
        ));
    }

    Ok(())
}

fn run_decode(state: &mut AppState) -> anyhow::Result<()> {
    let out_tar = state.decode.out_tar.as_ref().ok_or_else(|| anyhow::anyhow!("Output .tar not set"))?;

    let frames_dir = if let Some(frames) = state.decode.input_frames.as_ref() {
        frames.clone()
    } else {
        return Err(anyhow::anyhow!("Choose a frames folder (GUI MKV decode not wired yet)"));
    };

    let bytes = sllv_core::raster::decode_frames_dir_to_bytes_with_params(&frames_dir, &state.decode.rp)?;
    std::fs::write(out_tar, bytes)?;
    Ok(())
}
