use crate::fec::FecParams;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum Profile {
    Archive,
    Scan,
}

impl Profile {
    pub fn name(&self) -> &'static str {
        match self {
            Profile::Archive => "archive",
            Profile::Scan => "scan",
        }
    }

    pub fn defaults(&self) -> crate::raster::RasterParams {
        match self {
            Profile::Archive => {
                let mut p = crate::raster::RasterParams::default();
                p.cell_px = 2;
                p.border_cells = 2;
                p.fiducial_size_cells = 12;
                p.deskew = false; // archive expects exact pixels
                p.fec = None;
                p
            }
            Profile::Scan => {
                let mut p = crate::raster::RasterParams::default();
                p.cell_px = 6; // larger for camera robustness
                p.border_cells = 4;
                p.fiducial_size_cells = 18;
                p.deskew = true;
                p.fec = Some(FecParams {
                    data_shards: 12,
                    parity_shards: 12,
                    shard_bytes: 768,
                });
                p
            }
        }
    }
}
