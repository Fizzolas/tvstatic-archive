use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct Rgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum Palette8 {
    Basic,
}

#[derive(Debug, Error)]
pub enum PaletteError {
    #[error("symbol out of range: {0}")]
    SymbolOutOfRange(u8),
}

impl Palette8 {
    pub fn id(&self) -> &'static str {
        match self {
            Palette8::Basic => "basic8",
        }
    }

    /// 8 high-separation colors (3 bits/symbol).
    ///
    /// Index mapping (0..=7):
    /// 0 black, 1 white, 2 red, 3 green, 4 blue, 5 cyan, 6 magenta, 7 yellow
    pub fn color(&self, symbol: u8) -> Result<Rgb8, PaletteError> {
        let c = match symbol {
            0 => Rgb8 { r: 0, g: 0, b: 0 },
            1 => Rgb8 { r: 255, g: 255, b: 255 },
            2 => Rgb8 { r: 255, g: 0, b: 0 },
            3 => Rgb8 { r: 0, g: 255, b: 0 },
            4 => Rgb8 { r: 0, g: 0, b: 255 },
            5 => Rgb8 { r: 0, g: 255, b: 255 },
            6 => Rgb8 { r: 255, g: 0, b: 255 },
            7 => Rgb8 { r: 255, g: 255, b: 0 },
            _ => return Err(PaletteError::SymbolOutOfRange(symbol)),
        };
        Ok(c)
    }

    pub fn symbol_from_rgb(&self, r: u8, g: u8, b: u8) -> u8 {
        // Exact matching for Increment 1a (lossless pipeline).
        // Later increments will use nearest-color classification for camera scanning.
        match (r, g, b) {
            (0, 0, 0) => 0,
            (255, 255, 255) => 1,
            (255, 0, 0) => 2,
            (0, 255, 0) => 3,
            (0, 0, 255) => 4,
            (0, 255, 255) => 5,
            (255, 0, 255) => 6,
            (255, 255, 0) => 7,
            _ => 0, // fallback; will be improved later
        }
    }
}
