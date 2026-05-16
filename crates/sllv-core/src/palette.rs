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

    /// 8 high-separation colours (3 bits/symbol).
    ///
    /// Index mapping (0..=7):
    /// 0 black, 1 white, 2 red, 3 green, 4 blue, 5 cyan, 6 magenta, 7 yellow
    pub fn color(&self, symbol: u8) -> Result<Rgb8, PaletteError> {
        let c = match symbol {
            0 => Rgb8 { r: 0,   g: 0,   b: 0   },
            1 => Rgb8 { r: 255, g: 255, b: 255 },
            2 => Rgb8 { r: 255, g: 0,   b: 0   },
            3 => Rgb8 { r: 0,   g: 255, b: 0   },
            4 => Rgb8 { r: 0,   g: 0,   b: 255 },
            5 => Rgb8 { r: 0,   g: 255, b: 255 },
            6 => Rgb8 { r: 255, g: 0,   b: 255 },
            7 => Rgb8 { r: 255, g: 255, b: 0   },
            _ => return Err(PaletteError::SymbolOutOfRange(symbol)),
        };
        Ok(c)
    }

    pub fn symbol_from_rgb_exact(&self, r: u8, g: u8, b: u8) -> Option<u8> {
        match (r, g, b) {
            (0,   0,   0  ) => Some(0),
            (255, 255, 255) => Some(1),
            (255, 0,   0  ) => Some(2),
            (0,   255, 0  ) => Some(3),
            (0,   0,   255) => Some(4),
            (0,   255, 255) => Some(5),
            (255, 0,   255) => Some(6),
            (255, 255, 0  ) => Some(7),
            _ => None,
        }
    }

    /// Classify an RGB sample to the nearest Basic8 palette symbol.
    ///
    /// The Basic8 palette places one colour at each corner of the RGB cube, so
    /// the nearest-neighbour decision boundary for every channel is simply 127.
    /// Three comparisons replace the previous loop of 21 multiply-and-adds that
    /// ran for every cell on every decoded frame.
    #[inline]
    pub fn symbol_from_rgb_nearest(&self, r: u8, g: u8, b: u8) -> u8 {
        let rb = (r > 127) as u8;
        let gb = (g > 127) as u8;
        let bb = (b > 127) as u8;
        // Encodes the cube corner directly: R-bit is bit 2 (value 4),
        // but the palette assigns them in the order above, so we match:
        match (rb, gb, bb) {
            (0, 0, 0) => 0, // black
            (1, 1, 1) => 1, // white
            (1, 0, 0) => 2, // red
            (0, 1, 0) => 3, // green
            (0, 0, 1) => 4, // blue
            (0, 1, 1) => 5, // cyan
            (1, 0, 1) => 6, // magenta
            (1, 1, 0) => 7, // yellow
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_matches_exact_colors() {
        let pal = Palette8::Basic;
        for sym in 0u8..8 {
            let c = pal.color(sym).unwrap();
            assert_eq!(
                pal.symbol_from_rgb_nearest(c.r, c.g, c.b),
                sym,
                "symbol {sym} round-trips through nearest classifier"
            );
        }
    }

    #[test]
    fn nearest_handles_noisy_samples() {
        let pal = Palette8::Basic;
        // Slightly noisy red (camera capture simulation)
        assert_eq!(pal.symbol_from_rgb_nearest(230, 20, 15), 2);
        // Slightly noisy blue
        assert_eq!(pal.symbol_from_rgb_nearest(10, 5, 240), 4);
        // Slightly noisy white
        assert_eq!(pal.symbol_from_rgb_nearest(200, 210, 195), 1);
        // Slightly noisy black
        assert_eq!(pal.symbol_from_rgb_nearest(30, 20, 25), 0);
    }
}
