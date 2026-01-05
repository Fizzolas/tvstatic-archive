use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct Rgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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

    pub fn symbol_from_rgb_exact(&self, r: u8, g: u8, b: u8) -> Option<u8> {
        match (r, g, b) {
            (0, 0, 0) => Some(0),
            (255, 255, 255) => Some(1),
            (255, 0, 0) => Some(2),
            (0, 255, 0) => Some(3),
            (0, 0, 255) => Some(4),
            (0, 255, 255) => Some(5),
            (255, 0, 255) => Some(6),
            (255, 255, 0) => Some(7),
            _ => None,
        }
    }

    pub fn symbol_from_rgb_nearest(&self, r: u8, g: u8, b: u8) -> u8 {
        let mut best = 0u8;
        let mut best_d = u32::MAX;
        for sym in 0u8..8u8 {
            let c = self.color(sym).unwrap();
            let dr = (c.r as i32 - r as i32) as i32;
            let dg = (c.g as i32 - g as i32) as i32;
            let db = (c.b as i32 - b as i32) as i32;
            let d = (dr * dr + dg * dg + db * db) as u32;
            if d < best_d {
                best_d = d;
                best = sym;
            }
        }
        best
    }
}
