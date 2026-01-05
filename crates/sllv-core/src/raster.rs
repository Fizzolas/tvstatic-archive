use crate::manifest::EncodeManifest;
use crate::palette::{Palette8, Rgb8};
use image::{ImageBuffer, Rgb};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
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
            sync_color_symbol: 1, // white
            calibration_frames: 1,
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

    let cells = (p.grid_w as usize) * (p.grid_h as usize);
    let bits = cells * 3;
    let bytes_per_frame = bits / 8;

    let payload_bytes = std::cmp::min(bytes_per_frame as u32, p.chunk_bytes) as usize;

    // 1) Write sync slate frames.
    for i in 0..p.sync_frames {
        let img = render_solid_frame(p, p.sync_color_symbol)?;
        let path = out_dir.join(format!("frame_{:06}.png", i));
        img.save(path)?;
    }

    // 2) Write calibration frame(s): palette strip + simple checker.
    for j in 0..p.calibration_frames {
        let idx = p.sync_frames + j;
        let img = render_calibration_frame(p)?;
        let path = out_dir.join(format!("frame_{:06}.png", idx));
        img.save(path)?;
    }

    // 3) Data frames.
    let mut data_frames = 0u32;
    for (i, chunk) in input_bytes.chunks(payload_bytes).enumerate() {
        let mut frame_payload = vec![0u8; payload_bytes];
        frame_payload[..chunk.len()].copy_from_slice(chunk);

        let img = render_frame(&frame_payload, p)?;
        let frame_index = p.sync_frames + p.calibration_frames + (i as u32);
        let path = out_dir.join(format!("frame_{:06}.png", frame_index));
        img.save(path)?;
        data_frames += 1;
    }

    let total_frames = p.sync_frames + p.calibration_frames + data_frames;

    let manifest = EncodeManifest {
        magic: EncodeManifest::MAGIC.to_string(),
        version: EncodeManifest::VERSION,

        file_name: file_name.to_string(),
        total_bytes: input_bytes.len() as u64,
        chunk_bytes: payload_bytes as u32,

        grid_w: p.grid_w,
        grid_h: p.grid_h,
        cell_px: p.cell_px,

        palette: p.palette.id().to_string(),
        sha256_hex,
        frames: total_frames,
    };

    let manifest_path = out_dir.join("manifest.json");
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    let meta = json!({
        "bytes_per_frame_theoretical": bytes_per_frame,
        "bytes_per_frame_used": payload_bytes,
        "cells": cells,
        "bits_per_cell": 3,
        "sync_frames": p.sync_frames,
        "calibration_frames": p.calibration_frames,
        "data_frames": data_frames
    });
    fs::write(out_dir.join("debug.json"), serde_json::to_vec_pretty(&meta)?)?;

    Ok(manifest)
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

    // Determine how many non-data frames exist by scanning leading frames:
    // - sync frames are solid color
    // - calibration frame contains palette strip
    // For Increment 3b we encode with RasterParams default, so use that.
    let p = RasterParams::default();
    let start_index = p.sync_frames + p.calibration_frames;

    let mut out = Vec::with_capacity(manifest.total_bytes as usize);

    // Read from start_index up to manifest.frames.
    for i in start_index..manifest.frames {
        let path = in_dir.join(format!("frame_{:06}.png", i));
        let bytes = decode_frame(&path, &manifest, palette)?;
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

fn render_frame(payload: &[u8], p: &RasterParams) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    let w_px = p.grid_w * p.cell_px;
    let h_px = p.grid_h * p.cell_px;

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(w_px, h_px);

    let mut bit_i = 0usize;
    for y in 0..p.grid_h {
        for x in 0..p.grid_w {
            let sym = read_3bits(payload, bit_i);
            bit_i += 3;

            let Rgb8 { r, g, b } = p.palette.color(sym).unwrap();
            paint_cell(&mut img, x, y, p.cell_px, r, g, b);
        }
    }

    Ok(img)
}

fn render_solid_frame(p: &RasterParams, symbol: u8) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    let w_px = p.grid_w * p.cell_px;
    let h_px = p.grid_h * p.cell_px;
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(w_px, h_px);
    let Rgb8 { r, g, b } = p.palette.color(symbol).unwrap();
    for y in 0..p.grid_h {
        for x in 0..p.grid_w {
            paint_cell(&mut img, x, y, p.cell_px, r, g, b);
        }
    }
    Ok(img)
}

fn render_calibration_frame(p: &RasterParams) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    // Simple calibration: top row stripes of each palette color + checkerboard below.
    let w_px = p.grid_w * p.cell_px;
    let h_px = p.grid_h * p.cell_px;
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(w_px, h_px);

    // Fill with black.
    for y in 0..p.grid_h {
        for x in 0..p.grid_w {
            paint_cell(&mut img, x, y, p.cell_px, 0, 0, 0);
        }
    }

    // Palette strip uses first 8*block_w cells across the top.
    let block_w = std::cmp::max(1, p.grid_w / 8);
    for sym in 0u8..8u8 {
        let Rgb8 { r, g, b } = p.palette.color(sym).unwrap();
        let x0 = (sym as u32) * block_w;
        let x1 = std::cmp::min(p.grid_w, x0 + block_w);
        for y in 0..std::cmp::min(4, p.grid_h) {
            for x in x0..x1 {
                paint_cell(&mut img, x, y, p.cell_px, r, g, b);
            }
        }
    }

    // Checkerboard region.
    for y in 4..p.grid_h {
        for x in 0..p.grid_w {
            let sym = if ((x ^ y) & 1) == 0 { 1 } else { 0 };
            let Rgb8 { r, g, b } = p.palette.color(sym).unwrap();
            paint_cell(&mut img, x, y, p.cell_px, r, g, b);
        }
    }

    Ok(img)
}

fn decode_frame(path: &Path, m: &EncodeManifest, palette: Palette8) -> Result<Vec<u8>, RasterError> {
    let dyn_img = image::open(path)?;
    let img = dyn_img.to_rgb8();

    let mut payload = vec![0u8; m.chunk_bytes as usize];
    let mut bit_i = 0usize;

    for y in 0..m.grid_h {
        for x in 0..m.grid_w {
            let px = x * m.cell_px;
            let py = y * m.cell_px;
            let p0 = img.get_pixel(px, py);
            let sym = palette.symbol_from_rgb(p0[0], p0[1], p0[2]);
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
