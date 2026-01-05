use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodeManifest {
    pub magic: String,
    pub version: u16,

    pub file_name: String,
    pub total_bytes: u64,
    pub chunk_bytes: u32,

    pub grid_w: u32,
    pub grid_h: u32,
    pub cell_px: u32,

    pub palette: String,
    pub sha256_hex: String,
    pub frames: u32,
}

pub type DecodeManifest = EncodeManifest;

impl EncodeManifest {
    pub const MAGIC: &'static str = "SLLV";
    pub const VERSION: u16 = 1;
}
