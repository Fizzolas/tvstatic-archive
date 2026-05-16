use serde::{Deserialize, Serialize};

/// Manifest written alongside every encoded frame directory.
///
/// Version history:
///   v1 — initial release
///   v2 — added fec_data_shards, fec_parity_shards, deskew fields so a decoder
///        can reconstruct the exact parameters used at encode time without the
///        user having to remember or re-supply them.
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

    /// Number of RS data shards used during FEC encode. 0 means FEC was disabled.
    #[serde(default)]
    pub fec_data_shards: u32,
    /// Number of RS parity shards. 0 means FEC was disabled.
    #[serde(default)]
    pub fec_parity_shards: u32,
    /// Whether deskew was enabled at encode time.
    #[serde(default)]
    pub deskew: bool,
}

pub type DecodeManifest = EncodeManifest;

impl EncodeManifest {
    pub const MAGIC: &'static str = "SLLV";
    /// Bump to 2 now that the manifest carries FEC/deskew metadata.
    pub const VERSION: u16 = 2;
}
