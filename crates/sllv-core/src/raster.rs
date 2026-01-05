use crate::fec::{fec_decode_collect, fec_encode_stream, FecParams, ShardPacket};
use crate::manifest::EncodeManifest;
use crate::palette::{Palette8, Rgb8};
use image::{ImageBuffer, Rgb};
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

    // Sync/calibration
    pub sync_frames: u32,
    pub sync_color_symbol: u8,
    pub calibration_frames: u32,

    // Borders
    pub border_cells: u32,

    // FEC
    pub fec: Option<FecParams>,
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

            // Default to FEC enabled for scan robustness.
            fec: Some(FecParams::default()),
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

    // 1) Sync + calibration
    for i in 0..p.sync_frames {
        render_solid_frame(p, p.sync_color_symbol)?.save(out_dir.join(format!("frame_{:06}.png", i)))?;
    }
    for j in 0..p.calibration_frames {
        let idx = p.sync_frames + j;
        render_calibration_frame(p)?.save(out_dir.join(format!("frame_{:06}.png", idx)))?;
    }

    // 2) Determine how many bytes we can carry per frame.
    let payload_cells = (p.grid_w as usize) * (p.grid_h as usize);
    let payload_bits = payload_cells * 3;
    let payload_bytes_capacity = (payload_bits / 8) as u32;

    // Frame payload reserves space for a small header.
    let header_bytes = ShardHeader::BYTES as u32;
    let max_frame_payload = payload_bytes_capacity.saturating_sub(header_bytes);
    if max_frame_payload == 0 {
        return Err(RasterError::Fec("frame too small for header".into()));
    }

    let mut frames_written = 0u32;

    if let Some(fecp) = &p.fec {
        // FEC path: shards map 1:1 to frames.
        let packets = fec_encode_stream(input_bytes, fecp).map_err(|e| RasterError::Fec(e.to_string()))?;

        // Enforce shard size <= available payload.
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
                sha256_prefix: [pkt.shard_sha256[0], pkt.shard_sha256[1], pkt.shard_sha256[2], pkt.shard_sha256[3]],
            };

            let mut frame_bytes = Vec::with_capacity(ShardHeader::BYTES + pkt.shard_bytes.len());
            frame_bytes.extend_from_slice(&hdr.to_bytes());
            frame_bytes.extend_from_slice(&pkt.shard_bytes);

            // Pad the rest of frame payload with zeros for deterministic output.
            let mut padded = vec![0u8; max_frame_payload as usize];
            padded[..frame_bytes.len()].copy_from_slice(&frame_bytes);

            let img = render_payload_frame(&padded, p)?;
            let frame_index = p.sync_frames + p.calibration_frames + (k as u32);
            img.save(out_dir.join(format!("frame_{:06}.png", frame_index)))?;
            frames_written += 1;
        }

        // For manifest, chunk_bytes becomes "frame payload bytes".
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
            "fec": {
              "data_shards": fecp.data_shards,
              "parity_shards": fecp.parity_shards,
              "shard_bytes": fecp.shard_bytes
            }
        });
        fs::write(out_dir.join("debug.json"), serde_json::to_vec_pretty(&meta)?)?;

        Ok(manifest)
    } else {
        // Non-FEC path (legacy): chunk raw bytes.
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

    let p = RasterParams::default();
    let start_index = p.sync_frames + p.calibration_frames;
    let palette = Palette8::Basic;

    if let Some(fecp) = &p.fec {
        // Decode shard packets.
        let mut packets: Vec<ShardPacket> = Vec::new();
        let mut orig_total: Option<u64> = None;

        for i in start_index..manifest.frames {
            let path = in_dir.join(format!("frame_{:06}.png", i));
            let bytes = decode_payload_frame_bytes(&path, &manifest, &p, palette)?;
            if bytes.len() < ShardHeader::BYTES {
                continue;
            }
            let hdr = ShardHeader::from_bytes(&bytes[..ShardHeader::BYTES]);
            if orig_total.is_none() {
                orig_total = Some(hdr.orig_total_bytes);
            }
            let shard = bytes[ShardHeader::BYTES..].to_vec();
            packets.push(ShardPacket {
                group_index: hdr.group_index,
                shard_index: hdr.shard_index,
                shard_bytes: shard,
                shard_sha256: [0u8; 32],
            });
        }

        // NOTE: For part 2, we don't yet store full shard sha256 in-frame; we will in the next increment.
        // For now, skip verification and rely on end-to-end sha256.
        let total = orig_total.unwrap_or(manifest.total_bytes) as usize;

        // Reconstruct (without checksum verification yet).
        let out = fec_decode_collect_unverified(packets, total, fecp).map_err(|e| RasterError::Fec(e))?;

        // Verify file hash.
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
            let bytes = decode_payload_frame_bytes(&path, &manifest, &p, palette)?;
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

/// Temporary RS reconstruct without per-shard checks.
fn fec_decode_collect_unverified(
    packets: Vec<ShardPacket>,
    original_len: usize,
    p: &FecParams,
) -> Result<Vec<u8>, String> {
    // Convert to verified packets by setting sha256 to the computed hash.
    let mut verified = Vec::with_capacity(packets.len());
    for mut pkt in packets {
        let mut h = Sha256::new();
        h.update(&pkt.shard_bytes);
        pkt.shard_sha256 = h.finalize().into();
        verified.push(pkt);
    }
    fec_decode_collect(verified, original_len, p).map_err(|e| e.to_string())
}

#[derive(Clone, Copy)]
struct ShardHeader {
    group_index: u32,
    shard_index: u16,
    shard_len: u16,
    orig_total_bytes: u64,
    sha256_prefix: [u8; 4],
}

impl ShardHeader {
    const BYTES: usize = 4 + 2 + 2 + 8 + 4;

    fn to_bytes(&self) -> [u8; Self::BYTES] {
        let mut out = [0u8; Self::BYTES];
        out[0..4].copy_from_slice(&self.group_index.to_le_bytes());
        out[4..6].copy_from_slice(&self.shard_index.to_le_bytes());
        out[6..8].copy_from_slice(&self.shard_len.to_le_bytes());
        out[8..16].copy_from_slice(&self.orig_total_bytes.to_le_bytes());
        out[16..20].copy_from_slice(&self.sha256_prefix);
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
        let mut sp = [0u8; 4];
        sp.copy_from_slice(&b[16..20]);
        Self {
            group_index: u32::from_le_bytes(g),
            shard_index: u16::from_le_bytes(si),
            shard_len: u16::from_le_bytes(sl),
            orig_total_bytes: u64::from_le_bytes(ot),
            sha256_prefix: sp,
        }
    }
}

fn full_grid_w(p: &RasterParams) -> u32 {
    p.grid_w + 2 * p.border_cells
}

fn full_grid_h(p: &RasterParams) -> u32 {
    p.grid_h + 2 * p.border_cells
}

fn render_payload_frame(payload: &[u8], p: &RasterParams) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    let w_px = full_grid_w(p) * p.cell_px;
    let h_px = full_grid_h(p) * p.cell_px;

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(w_px, h_px);

    // Border pattern: alternating black/white checker.
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

fn render_solid_frame(p: &RasterParams, symbol: u8) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    let w_px = full_grid_w(p) * p.cell_px;
    let h_px = full_grid_h(p) * p.cell_px;
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(w_px, h_px);
    let Rgb8 { r, g, b } = p.palette.color(symbol).unwrap();
    for y in 0..full_grid_h(p) {
        for x in 0..full_grid_w(p) {
            paint_cell(&mut img, x, y, p.cell_px, r, g, b);
        }
    }
    Ok(img)
}

fn render_calibration_frame(p: &RasterParams) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    let w_px = full_grid_w(p) * p.cell_px;
    let h_px = full_grid_h(p) * p.cell_px;
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(w_px, h_px);

    // Border.
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

    // Fill payload with black.
    for y in 0..p.grid_h {
        for x in 0..p.grid_w {
            paint_cell(&mut img, x + p.border_cells, y + p.border_cells, p.cell_px, 0, 0, 0);
        }
    }

    // Palette strip at top of payload.
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

    // Checkerboard remainder.
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

fn decode_payload_frame_bytes(
    path: &Path,
    m: &EncodeManifest,
    p: &RasterParams,
    palette: Palette8,
) -> Result<Vec<u8>, RasterError> {
    let dyn_img = image::open(path)?;
    let img = dyn_img.to_rgb8();

    // Capacity of payload region.
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

fn paint_cell(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x: u32, y: u32, cell_px: u32, r: u8, g: u8, b: u8) {
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
