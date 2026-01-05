pub mod manifest;
pub mod palette;
pub mod raster;

pub use manifest::{DecodeManifest, EncodeManifest};
pub use palette::{Palette8, PaletteError};
pub use raster::{decode_frames_dir_to_bytes, encode_bytes_to_frames_dir, RasterParams, RasterError};
