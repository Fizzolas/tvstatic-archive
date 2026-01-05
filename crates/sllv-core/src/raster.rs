use crate::fec::{fec_decode_collect, fec_encode_stream, FecParams, ShardPacket};
use crate::manifest::EncodeManifest;
use crate::palette::{Palette8, Rgb8};
use crate::warp::{homography_from_4, warp_perspective_nearest, Pt2};
use image::Rgb;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RasterParams {
    pub grid_w: u32,
    pub grid_h: u32,
    pub cell_px: u32,
    pub chunk_bytes: u32,
    pub palette: Palette8,

    pub sync_frames: u32,
    pub sync_color_symbol: u8,
    pub calibration_frames: u32,

    pub border_cells: u32,

    pub fiducial_size_cells: u32,

    pub fec: Option<FecParams>,

    pub deskew: bool,
}

impl Default for RasterParams {
    fn default() -> Self {
        Self {
            grid_w: 256,
            grid_h: 256,
            cell_px: 2,
            chunk_bytes: 24 * 1024,
            palette: Palette8::Basic,

            sync_frames: 30,
            sync_color_symbol: 1,
            calibration_frames: 1,

            border_cells: 2,

            fiducial_size_cells: 12,

            fec: Some(FecParams::default()),

            deskew: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum RasterError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("image: {0}")]
    Image(#[from] image::ImageError),
    #[error("manifest missing")]
    ManifestMissing,
    #[error("manifest invalid magic/version")]
    ManifestInvalid,
    #[error("sha256 mismatch")]
    ShaMismatch,
    #[error("fec: {0}")]
    Fec(String),
}

// (rest of file unchanged)
