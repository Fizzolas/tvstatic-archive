use eframe::egui;

mod state;
mod ui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "SLLV",
        native_options,
        Box::new(|_cc| Ok(Box::new(state::AppState::default()))),
    )
}
