pub mod manifest;
pub mod palette;
pub mod raster;
pub mod pack;
pub mod fec;

pub use manifest::{DecodeManifest, EncodeManifest};
pub use palette::{Palette8, PaletteError};
pub use raster::{decode_frames_dir_to_bytes, encode_bytes_to_frames_dir, RasterParams, RasterError};
pub use pack::{pack_path_to_tar_bytes, PackError};
pub use fec::{fec_encode_stream, fec_decode_collect, FecParams, FecError};
