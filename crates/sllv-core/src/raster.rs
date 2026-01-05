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

#[derive(Debug, Clone)]
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

    // Decode pre-processing
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

pub fn encode_bytes_to_frames_dir(
    input_bytes: &[u8],
    file_name: &str,
    out_dir: &Path,
    p: &RasterParams,
) -> Result<EncodeManifest, RasterError> {
    fs::create_dir_all(out_dir)?;

    let mut hasher = Sha256::new();
    hasher.update(input_bytes);
    let sha256_hex = hex::encode(hasher.finalize());

    for i in 0..p.sync_frames {
        render_solid_frame(p, p.sync_color_symbol)?.save(out_dir.join(format!("frame_{:06}.png", i)))?;
    }
    for j in 0..p.calibration_frames {
        let idx = p.sync_frames + j;
        render_calibration_frame(p)?.save(out_dir.join(format!("frame_{:06}.png", idx)))?;
    }

    let payload_cells = (p.grid_w as usize) * (p.grid_h as usize);
    let payload_bits = payload_cells * 3;
    let payload_bytes_capacity = (payload_bits / 8) as u32;

    let header_bytes = ShardHeader::BYTES as u32;
    let max_frame_payload = payload_bytes_capacity.saturating_sub(header_bytes);
    if max_frame_payload == 0 {
        return Err(RasterError::Fec("frame too small for header".into()));
    }

    let mut frames_written = 0u32;

    if let Some(fecp) = &p.fec {
        let packets = fec_encode_stream(input_bytes, fecp).map_err(|e| RasterError::Fec(e.to_string()))?;

        if fecp.shard_bytes as u32 > max_frame_payload {
            return Err(RasterError::Fec(format!(
                "fec shard_bytes {} exceeds frame payload capacity {}",
                fecp.shard_bytes, max_frame_payload
            )));
        }

        for (k, pkt) in packets.iter().enumerate() {
            let hdr = ShardHeader {
                group_index: pkt.group_index,
                shard_index: pkt.shard_index,
                shard_len: pkt.shard_bytes.len() as u16,
                orig_total_bytes: input_bytes.len() as u64,
                shard_sha256: pkt.shard_sha256,
                header_crc32: 0,
            }
            .with_crc();

            let mut frame_bytes = Vec::with_capacity(ShardHeader::BYTES + pkt.shard_bytes.len());
            frame_bytes.extend_from_slice(&hdr.to_bytes());
            frame_bytes.extend_from_slice(&pkt.shard_bytes);

            let mut padded = vec![0u8; max_frame_payload as usize];
            padded[..frame_bytes.len()].copy_from_slice(&frame_bytes);

            let img = render_payload_frame(&padded, p)?;
            let frame_index = p.sync_frames + p.calibration_frames + (k as u32);
            img.save(out_dir.join(format!("frame_{:06}.png", frame_index)))?;
            frames_written += 1;
        }

        let manifest = EncodeManifest {
            magic: EncodeManifest::MAGIC.to_string(),
            version: EncodeManifest::VERSION,
            file_name: file_name.to_string(),
            total_bytes: input_bytes.len() as u64,
            chunk_bytes: max_frame_payload,
            grid_w: p.grid_w,
            grid_h: p.grid_h,
            cell_px: p.cell_px,
            palette: p.palette.id().to_string(),
            sha256_hex,
            frames: p.sync_frames + p.calibration_frames + frames_written,
        };

        fs::write(out_dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest)?)?;

        let meta = json!({
            "payload_bytes_capacity": payload_bytes_capacity,
            "header_bytes": ShardHeader::BYTES,
            "frame_payload_bytes": max_frame_payload,
            "sync_frames": p.sync_frames,
            "calibration_frames": p.calibration_frames,
            "data_frames": frames_written,
            "border_cells": p.border_cells,
            "fiducial_size_cells": p.fiducial_size_cells,
            "deskew": p.deskew,
            "fec": {
              "data_shards": fecp.data_shards,
              "parity_shards": fecp.parity_shards,
              "shard_bytes": fecp.shard_bytes
            }
        });
        fs::write(out_dir.join("debug.json"), serde_json::to_vec_pretty(&meta)?)?;

        Ok(manifest)
    } else {
        let max_payload = std::cmp::min(max_frame_payload, p.chunk_bytes) as usize;
        for (i, chunk) in input_bytes.chunks(max_payload).enumerate() {
            let mut frame_payload = vec![0u8; max_payload];
            frame_payload[..chunk.len()].copy_from_slice(chunk);
            let img = render_payload_frame(&frame_payload, p)?;
            let frame_index = p.sync_frames + p.calibration_frames + (i as u32);
            img.save(out_dir.join(format!("frame_{:06}.png", frame_index)))?;
            frames_written += 1;
        }

        let manifest = EncodeManifest {
            magic: EncodeManifest::MAGIC.to_string(),
            version: EncodeManifest::VERSION,
            file_name: file_name.to_string(),
            total_bytes: input_bytes.len() as u64,
            chunk_bytes: max_payload as u32,
            grid_w: p.grid_w,
            grid_h: p.grid_h,
            cell_px: p.cell_px,
            palette: p.palette.id().to_string(),
            sha256_hex,
            frames: p.sync_frames + p.calibration_frames + frames_written,
        };
        fs::write(out_dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest)?)?;
        Ok(manifest)
    }
}

pub fn decode_frames_dir_to_bytes(in_dir: &Path) -> Result<Vec<u8>, RasterError> {
    let manifest_path = in_dir.join("manifest.json");
    if !manifest_path.exists() {
        return Err(RasterError::ManifestMissing);
    }
    let manifest: EncodeManifest = serde_json::from_slice(&fs::read(manifest_path)?)?;
    if manifest.magic != EncodeManifest::MAGIC || manifest.version != EncodeManifest::VERSION {
        return Err(RasterError::ManifestInvalid);
    }

    let palette = Palette8::Basic;
    let p = RasterParams::default();
    let start_index = detect_data_start(in_dir, &manifest, &p, palette);

    if let Some(fecp) = &p.fec {
        let mut packets: Vec<ShardPacket> = Vec::new();
        let mut orig_total: Option<u64> = None;

        for i in start_index..manifest.frames {
            let path = in_dir.join(format!("frame_{:06}.png", i));
            let bytes = decode_frame_bytes_with_optional_deskew(&path, &manifest, &p, palette)?;

            if bytes.len() < ShardHeader::BYTES {
                continue;
            }
            let hdr = ShardHeader::from_bytes(&bytes[..ShardHeader::BYTES]);
            if !hdr.crc_ok(&bytes[..ShardHeader::BYTES]) {
                continue;
            }

            if orig_total.is_none() {
                orig_total = Some(hdr.orig_total_bytes);
            }

            let shard_end = ShardHeader::BYTES + (hdr.shard_len as usize);
            if shard_end > bytes.len() {
                continue;
            }

            let shard = bytes[ShardHeader::BYTES..shard_end].to_vec();

            let mut h = Sha256::new();
            h.update(&shard);
            let sha: [u8; 32] = h.finalize().into();
            if sha != hdr.shard_sha256 {
                continue;
            }

            packets.push(ShardPacket {
                group_index: hdr.group_index,
                shard_index: hdr.shard_index,
                shard_bytes: shard,
                shard_sha256: hdr.shard_sha256,
            });
        }

        let total = orig_total.unwrap_or(manifest.total_bytes) as usize;
        let out = fec_decode_collect(packets, total, fecp).map_err(|e| RasterError::Fec(e.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(&out);
        let sha256_hex = hex::encode(hasher.finalize());
        if sha256_hex != manifest.sha256_hex {
            return Err(RasterError::ShaMismatch);
        }
        Ok(out)
    } else {
        let mut out = Vec::with_capacity(manifest.total_bytes as usize);
        for i in start_index..manifest.frames {
            let path = in_dir.join(format!("frame_{:06}.png", i));
            let bytes = decode_frame_bytes_with_optional_deskew(&path, &manifest, &p, palette)?;
            out.extend_from_slice(&bytes);
        }
        out.truncate(manifest.total_bytes as usize);
        let mut hasher = Sha256::new();
        hasher.update(&out);
        let sha256_hex = hex::encode(hasher.finalize());
        if sha256_hex != manifest.sha256_hex {
            return Err(RasterError::ShaMismatch);
        }
        Ok(out)
    }
}

fn decode_frame_bytes_with_optional_deskew(
    path: &Path,
    m: &EncodeManifest,
    p: &RasterParams,
    palette: Palette8,
) -> Result<Vec<u8>, RasterError> {
    let dyn_img = image::open(path)?;
    let img = dyn_img.to_rgb8();

    let payload_img = if p.deskew {
        if let Some(warped) = deskew_with_fiducials(&img, m, p, palette) {
            warped
        } else {
            img
        }
    } else {
        img
    };

    decode_payload_from_rgb(&payload_img, m, p, palette)
}

fn deskew_with_fiducials(
    img: &image::ImageBuffer<Rgb<u8>, Vec<u8>>,
    m: &EncodeManifest,
    p: &RasterParams,
    palette: Palette8,
) -> Option<image::ImageBuffer<Rgb<u8>, Vec<u8>>> {
    // Detect each corner by scanning a window near that corner and finding pixels
    // closest to the expected fiducial symbol.

    let w = img.width();
    let h = img.height();

    let win = (p.fiducial_size_cells * p.cell_px * 3).min(w.min(h) / 2).max(32);

    let tl = find_color_centroid(img, 0, 0, win, win, 2, palette)?; // red
    let tr = find_color_centroid(img, w - win, 0, win, win, 3, palette)?; // green
    let bl = find_color_centroid(img, 0, h - win, win, win, 4, palette)?; // blue
    let br = find_color_centroid(img, w - win, h - win, win, win, 7, palette)?; // yellow

    // Destination: full canonical frame pixel dimensions (including border).
    let dst_w = (m.grid_w + 2 * p.border_cells) * m.cell_px;
    let dst_h = (m.grid_h + 2 * p.border_cells) * m.cell_px;

    let src_pts = [
        Pt2 { x: tl.x as f64, y: tl.y as f64 },
        Pt2 { x: tr.x as f64, y: tr.y as f64 },
        Pt2 { x: br.x as f64, y: br.y as f64 },
        Pt2 { x: bl.x as f64, y: bl.y as f64 },
    ];

    let dst_pts = [
        Pt2 { x: 0.0, y: 0.0 },
        Pt2 { x: (dst_w - 1) as f64, y: 0.0 },
        Pt2 { x: (dst_w - 1) as f64, y: (dst_h - 1) as f64 },
        Pt2 { x: 0.0, y: (dst_h - 1) as f64 },
    ];

    let hmat = homography_from_4(src_pts, dst_pts).ok()?;
    let warped = warp_perspective_nearest(img, &hmat, dst_w, dst_h).ok()?;

    Some(warped)
}

#[derive(Clone, Copy)]
struct CxCy {
    x: u32,
    y: u32,
}

fn find_color_centroid(
    img: &image::ImageBuffer<Rgb<u8>, Vec<u8>>,
    x0: u32,
    y0: u32,
    w: u32,
    h: u32,
    expected_symbol: u8,
    palette: Palette8,
) -> Option<CxCy> {
    let expected = palette.color(expected_symbol).ok()?;

    let mut sum_x = 0u64;
    let mut sum_y = 0u64;
    let mut n = 0u64;

    for y in y0..(y0 + h) {
        for x in x0..(x0 + w) {
            let p0 = img.get_pixel(x, y);
            let d = rgb_dist2(p0[0], p0[1], p0[2], expected.r, expected.g, expected.b);
            // loose threshold: allow quite a bit of camera drift
            if d < 60_000 {
                sum_x += x as u64;
                sum_y += y as u64;
                n += 1;
            }
        }
    }

    if n < 50 {
        return None;
    }

    Some(CxCy {
        x: (sum_x / n) as u32,
        y: (sum_y / n) as u32,
    })
}

fn rgb_dist2(r: u8, g: u8, b: u8, rr: u8, gg: u8, bb: u8) -> u32 {
    let dr = r as i32 - rr as i32;
    let dg = g as i32 - gg as i32;
    let db = b as i32 - bb as i32;
    (dr * dr + dg * dg + db * db) as u32
}

fn decode_payload_from_rgb(
    img: &image::ImageBuffer<Rgb<u8>, Vec<u8>>,
    m: &EncodeManifest,
    p: &RasterParams,
    palette: Palette8,
) -> Result<Vec<u8>, RasterError> {
    let payload_cells = (m.grid_w as usize) * (m.grid_h as usize);
    let payload_bits = payload_cells * 3;
    let payload_bytes = payload_bits / 8;

    let mut payload = vec![0u8; payload_bytes];
    let mut bit_i = 0usize;

    for y in 0..m.grid_h {
        for x in 0..m.grid_w {
            let gx = x + p.border_cells;
            let gy = y + p.border_cells;
            let px = gx * m.cell_px;
            let py = gy * m.cell_px;
            let p0 = img.get_pixel(px, py);

            let sym = palette.symbol_from_rgb_nearest(p0[0], p0[1], p0[2]);
            write_3bits(&mut payload, bit_i, sym);
            bit_i += 3;

            if (bit_i / 8) >= payload.len() {
                return Ok(payload);
            }
        }
    }

    Ok(payload)
}

fn detect_data_start(in_dir: &Path, m: &EncodeManifest, p: &RasterParams, palette: Palette8) -> u32 {
    let limit = std::cmp::min(m.frames, 300);
    let mut saw_cal = false;

    for i in 0..limit {
        let path = in_dir.join(format!("frame_{:06}.png", i));
        let Ok(stats) = frame_symbol_stats(&path, m, p, palette) else { continue };

        if stats.unique_symbols <= 1 {
            continue;
        }

        if !saw_cal {
            saw_cal = true;
            continue;
        }

        return i;
    }

    p.sync_frames + p.calibration_frames
}

struct SymbolStats {
    unique_symbols: usize,
}

fn frame_symbol_stats(path: &Path, m: &EncodeManifest, p: &RasterParams, palette: Palette8) -> Result<SymbolStats, RasterError> {
    let dyn_img = image::open(path)?;
    let img = dyn_img.to_rgb8();

    let mut seen = [false; 8];
    for y in 0..m.grid_h {
        for x in 0..m.grid_w {
            let gx = x + p.border_cells;
            let gy = y + p.border_cells;
            let px = gx * m.cell_px;
            let py = gy * m.cell_px;
            let p0 = img.get_pixel(px, py);
            let sym = palette.symbol_from_rgb_nearest(p0[0], p0[1], p0[2]) as usize;
            if sym < 8 {
                seen[sym] = true;
            }
        }
    }

    Ok(SymbolStats {
        unique_symbols: seen.iter().filter(|x| **x).count(),
    })
}

#[derive(Clone, Copy)]
struct ShardHeader {
    group_index: u32,
    shard_index: u16,
    shard_len: u16,
    orig_total_bytes: u64,
    shard_sha256: [u8; 32],
    header_crc32: u32,
}

impl ShardHeader {
    const BYTES: usize = 4 + 2 + 2 + 8 + 32 + 4;

    fn with_crc(mut self) -> Self {
        self.header_crc32 = crc32fast::hash(&self.to_bytes_no_crc());
        self
    }

    fn to_bytes_no_crc(&self) -> [u8; Self::BYTES - 4] {
        let mut out = [0u8; Self::BYTES - 4];
        out[0..4].copy_from_slice(&self.group_index.to_le_bytes());
        out[4..6].copy_from_slice(&self.shard_index.to_le_bytes());
        out[6..8].copy_from_slice(&self.shard_len.to_le_bytes());
        out[8..16].copy_from_slice(&self.orig_total_bytes.to_le_bytes());
        out[16..48].copy_from_slice(&self.shard_sha256);
        out
    }

    fn to_bytes(&self) -> [u8; Self::BYTES] {
        let mut out = [0u8; Self::BYTES];
        out[0..Self::BYTES - 4].copy_from_slice(&self.to_bytes_no_crc());
        out[Self::BYTES - 4..Self::BYTES].copy_from_slice(&self.header_crc32.to_le_bytes());
        out
    }

    fn from_bytes(b: &[u8]) -> Self {
        let mut g = [0u8; 4];
        g.copy_from_slice(&b[0..4]);
        let mut si = [0u8; 2];
        si.copy_from_slice(&b[4..6]);
        let mut sl = [0u8; 2];
        sl.copy_from_slice(&b[6..8]);
        let mut ot = [0u8; 8];
        ot.copy_from_slice(&b[8..16]);
        let mut sh = [0u8; 32];
        sh.copy_from_slice(&b[16..48]);
        let mut crc = [0u8; 4];
        crc.copy_from_slice(&b[48..52]);
        Self {
            group_index: u32::from_le_bytes(g),
            shard_index: u16::from_le_bytes(si),
            shard_len: u16::from_le_bytes(sl),
            orig_total_bytes: u64::from_le_bytes(ot),
            shard_sha256: sh,
            header_crc32: u32::from_le_bytes(crc),
        }
    }

    fn crc_ok(&self, raw: &[u8]) -> bool {
        if raw.len() < Self::BYTES {
            return false;
        }
        let calc = crc32fast::hash(&raw[..Self::BYTES - 4]);
        calc == self.header_crc32
    }
}

fn full_grid_w(p: &RasterParams) -> u32 {
    p.grid_w + 2 * p.border_cells
}

fn full_grid_h(p: &RasterParams) -> u32 {
    p.grid_h + 2 * p.border_cells
}

fn render_payload_frame(payload: &[u8], p: &RasterParams) -> Result<image::ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    let w_px = full_grid_w(p) * p.cell_px;
    let h_px = full_grid_h(p) * p.cell_px;

    let mut img: image::ImageBuffer<Rgb<u8>, Vec<u8>> = image::ImageBuffer::new(w_px, h_px);

    for y in 0..full_grid_h(p) {
        for x in 0..full_grid_w(p) {
            let in_payload = x >= p.border_cells
                && y >= p.border_cells
                && x < p.border_cells + p.grid_w
                && y < p.border_cells + p.grid_h;
            if !in_payload {
                let sym = if ((x ^ y) & 1) == 0 { 0 } else { 1 };
                let Rgb8 { r, g, b } = p.palette.color(sym).unwrap();
                paint_cell(&mut img, x, y, p.cell_px, r, g, b);
            }
        }
    }

    draw_corner_fiducials(&mut img, p);

    let mut bit_i = 0usize;
    for y in 0..p.grid_h {
        for x in 0..p.grid_w {
            let sym = read_3bits(payload, bit_i);
            bit_i += 3;
            let Rgb8 { r, g, b } = p.palette.color(sym).unwrap();
            paint_cell(
                &mut img,
                x + p.border_cells,
                y + p.border_cells,
                p.cell_px,
                r,
                g,
                b,
            );
        }
    }

    Ok(img)
}

fn draw_corner_fiducials(img: &mut image::ImageBuffer<Rgb<u8>, Vec<u8>>, p: &RasterParams) {
    let s = p.fiducial_size_cells;
    let b = p.border_cells;

    draw_l(img, b, b, s, p.cell_px, 2);
    draw_l(img, b + p.grid_w - s, b, s, p.cell_px, 3);
    draw_l(img, b, b + p.grid_h - s, s, p.cell_px, 4);
    draw_l(img, b + p.grid_w - s, b + p.grid_h - s, s, p.cell_px, 7);
}

fn draw_l(img: &mut image::ImageBuffer<Rgb<u8>, Vec<u8>>, x0: u32, y0: u32, s: u32, cell_px: u32, sym: u8) {
    let c = Palette8::Basic.color(sym).unwrap();

    for y in y0..(y0 + s) {
        paint_cell(img, x0, y, cell_px, c.r, c.g, c.b);
        paint_cell(img, x0 + 1, y, cell_px, c.r, c.g, c.b);
    }

    for x in x0..(x0 + s) {
        paint_cell(img, x, y0, cell_px, c.r, c.g, c.b);
        paint_cell(img, x, y0 + 1, cell_px, c.r, c.g, c.b);
    }
}

fn render_solid_frame(p: &RasterParams, symbol: u8) -> Result<image::ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    let w_px = full_grid_w(p) * p.cell_px;
    let h_px = full_grid_h(p) * p.cell_px;
    let mut img: image::ImageBuffer<Rgb<u8>, Vec<u8>> = image::ImageBuffer::new(w_px, h_px);
    let Rgb8 { r, g, b } = p.palette.color(symbol).unwrap();
    for y in 0..full_grid_h(p) {
        for x in 0..full_grid_w(p) {
            paint_cell(&mut img, x, y, p.cell_px, r, g, b);
        }
    }
    Ok(img)
}

fn render_calibration_frame(p: &RasterParams) -> Result<image::ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    let w_px = full_grid_w(p) * p.cell_px;
    let h_px = full_grid_h(p) * p.cell_px;
    let mut img: image::ImageBuffer<Rgb<u8>, Vec<u8>> = image::ImageBuffer::new(w_px, h_px);

    for y in 0..full_grid_h(p) {
        for x in 0..full_grid_w(p) {
            let in_payload = x >= p.border_cells
                && y >= p.border_cells
                && x < p.border_cells + p.grid_w
                && y < p.border_cells + p.grid_h;
            if !in_payload {
                let sym = if ((x ^ y) & 1) == 0 { 0 } else { 1 };
                let Rgb8 { r, g, b } = p.palette.color(sym).unwrap();
                paint_cell(&mut img, x, y, p.cell_px, r, g, b);
            }
        }
    }

    draw_corner_fiducials(&mut img, p);

    for y in 0..p.grid_h {
        for x in 0..p.grid_w {
            paint_cell(&mut img, x + p.border_cells, y + p.border_cells, p.cell_px, 0, 0, 0);
        }
    }

    let block_w = std::cmp::max(1, p.grid_w / 8);
    for sym in 0u8..8u8 {
        let Rgb8 { r, g, b } = p.palette.color(sym).unwrap();
        let x0 = (sym as u32) * block_w;
        let x1 = std::cmp::min(p.grid_w, x0 + block_w);
        for y in 0..std::cmp::min(4, p.grid_h) {
            for x in x0..x1 {
                paint_cell(
                    &mut img,
                    x + p.border_cells,
                    y + p.border_cells,
                    p.cell_px,
                    r,
                    g,
                    b,
                );
            }
        }
    }

    for y in 4..p.grid_h {
        for x in 0..p.grid_w {
            let sym = if ((x ^ y) & 1) == 0 { 1 } else { 0 };
            let Rgb8 { r, g, b } = p.palette.color(sym).unwrap();
            paint_cell(
                &mut img,
                x + p.border_cells,
                y + p.border_cells,
                p.cell_px,
                r,
                g,
                b,
            );
        }
    }

    Ok(img)
}

fn paint_cell(img: &mut image::ImageBuffer<Rgb<u8>, Vec<u8>>, x: u32, y: u32, cell_px: u32, r: u8, g: u8, b: u8) {
    let x0 = x * cell_px;
    let y0 = y * cell_px;
    for dy in 0..cell_px {
        for dx in 0..cell_px {
            img.put_pixel(x0 + dx, y0 + dy, Rgb([r, g, b]));
        }
    }
}

fn read_3bits(bytes: &[u8], bit_i: usize) -> u8 {
    let mut v = 0u8;
    for k in 0..3 {
        let i = bit_i + k;
        let b = bytes.get(i / 8).copied().unwrap_or(0);
        let bit = (b >> (i % 8)) & 1;
        v |= (bit as u8) << k;
    }
    v
}

fn write_3bits(bytes: &mut [u8], bit_i: usize, sym: u8) {
    for k in 0..3 {
        let i = bit_i + k;
        let byte_i = i / 8;
        if byte_i >= bytes.len() {
            return;
        }
        let bit = (sym >> k) & 1;
        let mask = 1u8 << (i % 8);
        if bit == 1 {
            bytes[byte_i] |= mask;
        } else {
            bytes[byte_i] &= !mask;
        }
    }
}
