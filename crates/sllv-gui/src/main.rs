use crate::state::AppState;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "SLLV GUI",
        native_options,
        Box::new(|_cc| Ok(Box::new(AppState::default()))),
    );
}
