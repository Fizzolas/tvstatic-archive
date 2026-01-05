use thiserror::Error;

#[derive(Debug, Error)]
pub enum PaletteError {
    #[error("invalid palette id")]
    InvalidPalette,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum Palette8 {
    Basic,
}

#[derive(Debug, Clone, Copy)]
pub struct Rgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
