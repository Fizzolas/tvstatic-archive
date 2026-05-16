use crate::{fec::FecParams, palette::Palette8, raster::RasterParams};

/// Encode/decode profile selection.
///
/// # Reed-Solomon shard limit
/// The RS implementation uses GF(2^8), which supports at most **256 total shards**
/// (data + parity combined). Both profiles below are well within this limit, but
/// if you ever bump `data_shards` or `parity_shards` in `Scan`, ensure the sum
/// stays <= 256. `fec_encode_stream` enforces this at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// Lossless / exact-pixel path. No deskew, no FEC. Best for PNG frame
    /// archives or truly lossless video containers (FFV1-MKV).
    Archive,
    /// Robust path for camera / projected-screen pipelines. Enables perspective
    /// deskew and Reed-Solomon FEC so that a minority of corrupted or missing
    /// frames can be recovered automatically.
    Scan,
}

impl Profile {
    pub fn defaults(&self) -> RasterParams {
        match self {
            Profile::Archive => RasterParams {
                grid_w: 256,
                grid_h: 256,
                cell_px: 2,
                chunk_bytes: 0,
                palette: Palette8::Basic,
                sync_frames: 30,
                sync_color_symbol: 1,
                calibration_frames: 1,
                border_cells: 2,
                fiducial_size_cells: 12,
                deskew: false,
                fec: None,
            },
            Profile::Scan => {
                // data_shards(12) + parity_shards(12) = 24, well within GF(2^8) limit of 256
                RasterParams {
                    grid_w: 256,
                    grid_h: 256,
                    cell_px: 6,
                    chunk_bytes: 0,
                    palette: Palette8::Basic,
                    sync_frames: 30,
                    sync_color_symbol: 1,
                    calibration_frames: 1,
                    border_cells: 2,
                    fiducial_size_cells: 12,
                    deskew: true,
                    fec: Some(FecParams {
                        data_shards: 12,
                        parity_shards: 12,
                        shard_bytes: 768,
                    }),
                }
            }
        }
    }
}
