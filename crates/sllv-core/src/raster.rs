use crate::fec::{fec_decode_collect, fec_encode_stream, FecParams, ShardPacket};
use crate::manifest::EncodeManifest;
use crate::palette::Palette8;
use crate::warp::{homography_from_4, warp_perspective_bilinear, Pt2};
use image::Rgb;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;
use std::sync::mpsc;
use std::sync::Arc;
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

#[derive(Debug, Clone)]
pub enum ProgressMsg {
    Stage { name: String, done: u64, total: u64 },
    Done,
    Error(String),
}

pub fn encode_bytes_to_frames_dir(
    input_bytes: &[u8],
    file_name: &str,
    out_dir: &Path,
    p: &RasterParams,
) -> Result<EncodeManifest, RasterError> {
    encode_bytes_to_frames_dir_with_progress(input_bytes, file_name, out_dir, p, None)
}

pub fn encode_bytes_to_frames_dir_with_progress(
    input_bytes: &[u8],
    file_name: &str,
    out_dir: &Path,
    p: &RasterParams,
    progress_tx: Option<mpsc::Sender<ProgressMsg>>,
) -> Result<EncodeManifest, RasterError> {
    fs::create_dir_all(out_dir)?;

    let mut hasher = Sha256::new();
    hasher.update(input_bytes);
    let sha256_hex = hex::encode(hasher.finalize());

    // Sync frames
    for i in 0..p.sync_frames {
        render_solid_frame(p, p.sync_color_symbol)?.save(out_dir.join(format!("frame_{:06}.png", i)))?;
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(ProgressMsg::Stage {
                name: "sync".into(),
                done: (i + 1) as u64,
                total: p.sync_frames as u64,
            });
        }
    }

    // Calibration frames
    for j in 0..p.calibration_frames {
        let idx = p.sync_frames + j;
        render_calibration_frame(p)?.save(out_dir.join(format!("frame_{:06}.png", idx)))?;
        if let Some(ref tx) = progress_tx {
            let _ = tx.send(ProgressMsg::Stage {
                name: "calibration".into(),
                done: (j + 1) as u64,
                total: p.calibration_frames as u64,
            });
        }
    }

    let payload_cells = (p.grid_w as usize) * (p.grid_h as usize);
    let payload_bits = payload_cells * 3;
    let payload_bytes_capacity = (payload_bits / 8) as u32;

    let header_bytes = ShardHeader::BYTES as u32;
    let max_frame_payload = if p.fec.is_some() {
        payload_bytes_capacity.saturating_sub(header_bytes)
    } else {
        payload_bytes_capacity
    };

    if max_frame_payload == 0 {
        return Err(RasterError::Fec("frame too small for payload".into()));
    }

    let mut frames_written = 0u32;

    if let Some(fecp) = &p.fec {
        if input_bytes.is_empty() {
            let manifest = EncodeManifest {
                magic: EncodeManifest::MAGIC.to_string(),
                version: EncodeManifest::VERSION,
                file_name: file_name.to_string(),
                total_bytes: 0,
                chunk_bytes: max_frame_payload,
                grid_w: p.grid_w,
                grid_h: p.grid_h,
                cell_px: p.cell_px,
                palette: p.palette.id().to_string(),
                sha256_hex,
                frames: p.sync_frames + p.calibration_frames,
                fec_data_shards: fecp.data_shards as u32,
                fec_parity_shards: fecp.parity_shards as u32,
                deskew: p.deskew,
            };
            fs::write(out_dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest)?)?;
            return Ok(manifest);
        }

        if fecp.shard_bytes as u32 > max_frame_payload {
            return Err(RasterError::Fec(format!(
                "fec shard_bytes {} exceeds frame payload capacity {} — \
                 reduce shard_bytes or increase grid size",
                fecp.shard_bytes, max_frame_payload
            )));
        }

        let packets = fec_encode_stream(input_bytes, fecp).map_err(|e| RasterError::Fec(e.to_string()))?;

        let total_packets = packets.len() as u64;
        let max_payload = max_frame_payload;
        let orig_total_bytes = input_bytes.len() as u64;

        let (tx_img, rx_img) = mpsc::sync_channel::<(u32, image::ImageBuffer<Rgb<u8>, Vec<u8>>)>(32);
        let packets_arc = Arc::new(packets);
        let p_clone = p.clone();

        let num_workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8);
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        std::thread::scope(|s| {
            for _ in 0..num_workers {
                let tx = tx_img.clone();
                let pkts = Arc::clone(&packets_arc);
                let params = p_clone.clone();
                let counter = Arc::clone(&counter);
                s.spawn(move || {
                    loop {
                        let idx = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if idx >= pkts.len() { break; }
                        let pkt = &pkts[idx];

                        let hdr = ShardHeader {
                            group_index: pkt.group_index,
                            shard_index: pkt.shard_index,
                            shard_len: pkt.shard_bytes.len() as u16,
                            orig_total_bytes,
                            shard_sha256: pkt.shard_sha256,
                            header_crc32: 0,
                        }
                        .with_crc();

                        let mut frame_bytes = Vec::with_capacity(ShardHeader::BYTES + pkt.shard_bytes.len());
                        frame_bytes.extend_from_slice(&hdr.to_bytes());
                        frame_bytes.extend_from_slice(&pkt.shard_bytes);

                        let mut padded = vec![0u8; max_payload as usize];
                        padded[..frame_bytes.len()].copy_from_slice(&frame_bytes);

                        if let Ok(img) = render_payload_frame(&padded, &params) {
                            let _ = tx.send((idx as u32, img));
                        }
                    }
                });
            }
            drop(tx_img);

            use std::collections::BTreeMap;
            let mut pending: BTreeMap<u32, image::ImageBuffer<Rgb<u8>, Vec<u8>>> = BTreeMap::new();
            let mut next_to_write: u32 = 0;

            for (pkt_idx, img) in rx_img {
                pending.insert(pkt_idx, img);
                while let Some(frame_img) = pending.remove(&next_to_write) {
                    let frame_index = p_clone.sync_frames + p_clone.calibration_frames + next_to_write;
                    frame_img.save(out_dir.join(format!("frame_{:06}.png", frame_index))).ok();
                    frames_written += 1;
                    next_to_write += 1;

                    if let Some(ref tx) = progress_tx {
                        let _ = tx.send(ProgressMsg::Stage {
                            name: "encode".into(),
                            done: frames_written as u64,
                            total: total_packets,
                        });
                    }
                }
            }

            for (_, frame_img) in pending {
                let frame_index = p_clone.sync_frames + p_clone.calibration_frames + next_to_write;
                frame_img.save(out_dir.join(format!("frame_{:06}.png", frame_index))).ok();
                frames_written += 1;
                next_to_write += 1;
            }
        });

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
            fec_data_shards: fecp.data_shards as u32,
            fec_parity_shards: fecp.parity_shards as u32,
            deskew: p.deskew,
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
            "fec": p.fec.as_ref().map(|fecp| json!({
              "data_shards": fecp.data_shards,
              "parity_shards": fecp.parity_shards,
              "shard_bytes": fecp.shard_bytes
            }))
        });
        fs::write(out_dir.join("debug.json"), serde_json::to_vec_pretty(&meta)?)?;

        Ok(manifest)
    } else {
        let max_payload = std::cmp::min(max_frame_payload, p.chunk_bytes) as usize;
        // FIX: manual_checked_division + manual_div_ceil → use .div_ceil()
        let total_chunks = input_bytes.len().div_ceil(max_payload);

        for (i, chunk) in input_bytes.chunks(max_payload).enumerate() {
            let mut frame_payload = vec![0u8; max_payload];
            frame_payload[..chunk.len()].copy_from_slice(chunk);
            let img = render_payload_frame(&frame_payload, p)?;
            let frame_index = p.sync_frames + p.calibration_frames + (i as u32);
            img.save(out_dir.join(format!("frame_{:06}.png", frame_index)))?;
            frames_written += 1;

            if let Some(ref tx) = progress_tx {
                let _ = tx.send(ProgressMsg::Stage {
                    name: "encode".into(),
                    done: frames_written as u64,
                    total: total_chunks as u64,
                });
            }
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
            fec_data_shards: 0,
            fec_parity_shards: 0,
            deskew: p.deskew,
        };
        fs::write(out_dir.join("manifest.json"), serde_json::to_vec_pretty(&manifest)?)?;
        Ok(manifest)
    }
}

pub fn decode_frames_dir_to_bytes(in_dir: &Path) -> Result<Vec<u8>, RasterError> {
    decode_frames_dir_to_bytes_with_params(in_dir, &RasterParams::default())
}

pub fn decode_frames_dir_to_bytes_with_params(in_dir: &Path, p: &RasterParams) -> Result<Vec<u8>, RasterError> {
    decode_frames_dir_to_bytes_with_progress(in_dir, p, None)
}

pub fn decode_frames_dir_to_bytes_with_progress(
    in_dir: &Path,
    p: &RasterParams,
    progress_tx: Option<mpsc::Sender<ProgressMsg>>,
) -> Result<Vec<u8>, RasterError> {
    let manifest_path = in_dir.join("manifest.json");
    if !manifest_path.exists() {
        return Err(RasterError::ManifestMissing);
    }
    let manifest: EncodeManifest = serde_json::from_slice(&fs::read(manifest_path)?)?;
    if manifest.magic != EncodeManifest::MAGIC || manifest.version != EncodeManifest::VERSION {
        return Err(RasterError::ManifestInvalid);
    }

    let palette = Palette8::Basic;
    let start_index = detect_data_start(&manifest, p);

    let total_frames = (manifest.frames - start_index) as u64;

    if let Some(fecp) = &p.fec {
        let (tx_pkt, rx_pkt) = mpsc::sync_channel::<Option<ShardPacket>>(16);
        let in_dir_arc = Arc::new(in_dir.to_path_buf());
        let manifest_arc = Arc::new(manifest.clone());
        let p_arc = Arc::new(p.clone());

        let num_workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(8);
        let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));

        std::thread::scope(|s| {
            for _ in 0..num_workers {
                let tx = tx_pkt.clone();
                let dir = Arc::clone(&in_dir_arc);
                let m = Arc::clone(&manifest_arc);
                let params = Arc::clone(&p_arc);
                let counter = Arc::clone(&counter);
                s.spawn(move || {
                    loop {
                        let idx = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let i = start_index + idx;
                        if i >= m.frames { break; }

                        let path = dir.join(format!("frame_{:06}.png", i));
                        let mut out_pkt: Option<ShardPacket> = None;

                        if let Ok(bytes) = decode_frame_bytes_with_optional_deskew(&path, &m, &params, palette) {
                            if bytes.len() >= ShardHeader::BYTES {
                                let hdr = ShardHeader::from_bytes(&bytes[..ShardHeader::BYTES]);
                                if hdr.crc_ok(&bytes[..ShardHeader::BYTES]) {
                                    let shard_end = ShardHeader::BYTES + (hdr.shard_len as usize);
                                    if shard_end <= bytes.len() {
                                        let shard = bytes[ShardHeader::BYTES..shard_end].to_vec();
                                        let mut h = Sha256::new();
                                        h.update(&shard);
                                        let sha: [u8; 32] = h.finalize().into();
                                        if sha == hdr.shard_sha256 {
                                            out_pkt = Some(ShardPacket {
                                                group_index: hdr.group_index,
                                                shard_index: hdr.shard_index,
                                                shard_bytes: shard,
                                                shard_sha256: hdr.shard_sha256,
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        let _ = tx.send(out_pkt);
                    }
                });
            }
            drop(tx_pkt);

            let mut packets = Vec::new();
            // FIX: explicit_counter_loop → use .enumerate() on the iterator
            for (decoded, maybe_pkt) in rx_pkt.into_iter().enumerate() {
                if let Some(pkt) = maybe_pkt {
                    packets.push(pkt);
                }
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(ProgressMsg::Stage {
                        name: "decode".into(),
                        done: (decoded + 1) as u64,
                        total: total_frames,
                    });
                }
            }

            let total = manifest.total_bytes as usize;
            let out = fec_decode_collect(packets, total, fecp).map_err(|e| RasterError::Fec(format!("{e:?}")))?;

            let mut hasher = Sha256::new();
            hasher.update(&out);
            let sha256_hex = hex::encode(hasher.finalize());
            if sha256_hex != manifest.sha256_hex {
                return Err(RasterError::ShaMismatch);
            }
            Ok(out)
        })
    } else {
        let per_frame = manifest.chunk_bytes as usize;
        let mut out = Vec::with_capacity(manifest.total_bytes as usize);

        for i in start_index..manifest.frames {
            if out.len() >= manifest.total_bytes as usize { break; }
            let path = in_dir.join(format!("frame_{:06}.png", i));
            if let Ok(bytes) = decode_frame_bytes_with_optional_deskew(&path, &manifest, p, palette) {
                let remaining = manifest.total_bytes as usize - out.len();
                let take = remaining.min(bytes.len()).min(per_frame);
                out.extend_from_slice(&bytes[..take]);
            }
            if let Some(ref tx) = progress_tx {
                let done = (i - start_index + 1) as u64;
                let _ = tx.send(ProgressMsg::Stage {
                    name: "decode".into(),
                    done,
                    total: total_frames,
                });
            }
        }

        let mut hasher = Sha256::new();
        hasher.update(&out);
        let sha256_hex = hex::encode(hasher.finalize());
        if sha256_hex != manifest.sha256_hex {
            return Err(RasterError::ShaMismatch);
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// Private helpers below — pixel rendering, frame decode, etc.
// ---------------------------------------------------------------------------

struct ShardHeader {
    group_index: u32,
    shard_index: u16,
    shard_len: u16,
    orig_total_bytes: u64,
    shard_sha256: [u8; 32],
    header_crc32: u32,
}

impl ShardHeader {
    // 4 + 2 + 2 + 8 + 32 + 4 = 52
    const BYTES: usize = 52;

    fn to_bytes(&self) -> [u8; 52] {
        let mut b = [0u8; 52];
        b[0..4].copy_from_slice(&self.group_index.to_le_bytes());
        b[4..6].copy_from_slice(&self.shard_index.to_le_bytes());
        b[6..8].copy_from_slice(&self.shard_len.to_le_bytes());
        b[8..16].copy_from_slice(&self.orig_total_bytes.to_le_bytes());
        b[16..48].copy_from_slice(&self.shard_sha256);
        b[48..52].copy_from_slice(&self.header_crc32.to_le_bytes());
        b
    }

    fn from_bytes(b: &[u8]) -> Self {
        let group_index      = u32::from_le_bytes(b[0..4].try_into().unwrap());
        let shard_index      = u16::from_le_bytes(b[4..6].try_into().unwrap());
        let shard_len        = u16::from_le_bytes(b[6..8].try_into().unwrap());
        let orig_total_bytes = u64::from_le_bytes(b[8..16].try_into().unwrap());
        let shard_sha256: [u8; 32] = b[16..48].try_into().unwrap();
        let header_crc32     = u32::from_le_bytes(b[48..52].try_into().unwrap());
        Self { group_index, shard_index, shard_len, orig_total_bytes, shard_sha256, header_crc32 }
    }

    fn with_crc(mut self) -> Self {
        let tmp = self.to_bytes();
        self.header_crc32 = crc32fast::hash(&tmp[..48]);
        self
    }

    fn crc_ok(&self, raw: &[u8]) -> bool {
        crc32fast::hash(&raw[..48]) == self.header_crc32
    }
}

/// Write a single colour to every cell of the grid.
fn render_solid_frame(
    p: &RasterParams,
    symbol: u8,
) -> Result<image::ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    let w = p.grid_w * p.cell_px;
    let h = p.grid_h * p.cell_px;
    let color = p.palette.color(symbol).map_err(|e| RasterError::Fec(e.to_string()))?;
    let mut img = image::ImageBuffer::new(w, h);
    for pixel in img.pixels_mut() {
        *pixel = Rgb([color.r, color.g, color.b]);
    }
    Ok(img)
}

/// Render the calibration / fiducial frame.
fn render_calibration_frame(
    p: &RasterParams,
) -> Result<image::ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    let w = p.grid_w * p.cell_px;
    let h = p.grid_h * p.cell_px;
    let mut img = image::ImageBuffer::new(w, h);

    let black = Rgb([0u8, 0, 0]);
    let white = Rgb([255u8, 255, 255]);

    for pixel in img.pixels_mut() {
        *pixel = white;
    }

    let b = p.border_cells * p.cell_px;
    for y in 0..h {
        for x in 0..w {
            if x < b || x >= w - b || y < b || y >= h - b {
                img.put_pixel(x, y, black);
            }
        }
    }

    let fid = p.fiducial_size_cells * p.cell_px;
    let corners = [
        (b, b),
        (w - b - fid, b),
        (b, h - b - fid),
        (w - b - fid, h - b - fid),
    ];
    for (cx, cy) in corners {
        for dy in 0..fid {
            for dx in 0..fid {
                img.put_pixel(cx + dx, cy + dy, black);
            }
        }
    }

    Ok(img)
}

/// Encode a byte slice into a grid frame.
fn render_payload_frame(
    bytes: &[u8],
    p: &RasterParams,
) -> Result<image::ImageBuffer<Rgb<u8>, Vec<u8>>, RasterError> {
    let w = p.grid_w * p.cell_px;
    let h = p.grid_h * p.cell_px;
    let mut img = image::ImageBuffer::new(w, h);

    let total_cells = (p.grid_w * p.grid_h) as usize;
    let mut symbols = vec![0u8; total_cells];
    write_3bits(bytes, &mut symbols);

    for (cell_idx, &sym) in symbols.iter().enumerate() {
        let cx = (cell_idx % p.grid_w as usize) as u32;
        let cy = (cell_idx / p.grid_w as usize) as u32;
        let color = p.palette.color(sym).map_err(|e| RasterError::Fec(e.to_string()))?;
        paint_cell(&mut img, cx, cy, p.cell_px, Rgb([color.r, color.g, color.b]));
    }

    Ok(img)
}

/// Paint a single grid cell (cell_px × cell_px pixels).
#[inline]
fn paint_cell(
    img: &mut image::ImageBuffer<Rgb<u8>, Vec<u8>>,
    cx: u32,
    cy: u32,
    cell_px: u32,
    color: Rgb<u8>,
) {
    let px0 = cx * cell_px;
    let py0 = cy * cell_px;
    for dy in 0..cell_px {
        for dx in 0..cell_px {
            img.put_pixel(px0 + dx, py0 + dy, color);
        }
    }
}

/// Pack `bytes` into 3-bit symbols (one per grid cell).
fn write_3bits(bytes: &[u8], symbols: &mut [u8]) {
    // FIX: needless_range_loop — iterate directly with enumerate
    for (si, slot) in symbols.iter_mut().enumerate() {
        let bit_pos = si * 3;
        let byte_idx = bit_pos / 8;
        let bit_off  = bit_pos % 8;
        let sym = if byte_idx < bytes.len() {
            let b0 = bytes[byte_idx] as u16;
            let b1 = if byte_idx + 1 < bytes.len() { bytes[byte_idx + 1] as u16 } else { 0 };
            let word = (b0 << 8) | b1;
            ((word >> (16 - 3 - bit_off)) & 0x07) as u8
        } else {
            0
        };
        *slot = sym;
    }
}

/// Extract 3-bit symbols from a decoded pixel grid back into bytes.
fn read_3bits(symbols: &[u8], out: &mut [u8]) {
    // FIX: needless_range_loop — iterate directly with enumerate
    for (bi, slot) in out.iter_mut().enumerate() {
        let mut byte_val: u8 = 0;
        for bit in 0..8 {
            let bit_pos = bi * 8 + bit;
            let si = bit_pos / 3;
            let bit_off = bit_pos % 3;
            let sym = if si < symbols.len() { symbols[si] } else { 0 };
            let bit_val = (sym >> (2 - bit_off)) & 1;
            byte_val = (byte_val << 1) | bit_val;
        }
        *slot = byte_val;
    }
}

/// Decode raw pixel bytes from a single frame PNG.
fn decode_frame_bytes(
    path: &Path,
    p: &RasterParams,
) -> Result<Vec<u8>, RasterError> {
    let img = image::open(path)?.into_rgb8();
    let total_cells = (p.grid_w * p.grid_h) as usize;
    let mut symbols = vec![0u8; total_cells];

    // FIX: needless_range_loop — iterate with enumerate
    for (cell_idx, slot) in symbols.iter_mut().enumerate() {
        let cx = (cell_idx % p.grid_w as usize) as u32;
        let cy = (cell_idx / p.grid_w as usize) as u32;
        let px = cx * p.cell_px + p.cell_px / 2;
        let py = cy * p.cell_px + p.cell_px / 2;
        let pixel = img.get_pixel(px, py);
        *slot = p.palette.symbol_from_rgb_nearest(pixel[0], pixel[1], pixel[2]);
    }

    let payload_cells = p.grid_w as usize * p.grid_h as usize;
    let payload_bits  = payload_cells * 3;
    let payload_bytes = payload_bits / 8;
    let mut out = vec![0u8; payload_bytes];
    read_3bits(&symbols, &mut out);
    Ok(out)
}

/// Optionally deskew a frame before decoding, depending on `p.deskew`.
fn decode_frame_bytes_with_optional_deskew(
    path: &Path,
    manifest: &EncodeManifest,
    p: &RasterParams,
    _palette: Palette8,
) -> Result<Vec<u8>, RasterError> {
    if p.deskew {
        let img = image::open(path)?.into_rgb8();
        let (iw, ih) = (img.width(), img.height());

        let dst_w = manifest.grid_w * manifest.cell_px;
        let dst_h = manifest.grid_h * manifest.cell_px;

        let src_pts = [
            Pt2 { x: 0.0,        y: 0.0 },
            Pt2 { x: iw as f64,  y: 0.0 },
            Pt2 { x: iw as f64,  y: ih as f64 },
            Pt2 { x: 0.0,        y: ih as f64 },
        ];
        let dst_pts = [
            Pt2 { x: 0.0,          y: 0.0 },
            Pt2 { x: dst_w as f64, y: 0.0 },
            Pt2 { x: dst_w as f64, y: dst_h as f64 },
            Pt2 { x: 0.0,          y: dst_h as f64 },
        ];

        let h = homography_from_4(src_pts, dst_pts)
            .map_err(|e| RasterError::Fec(format!("warp: {e:?}")))?;
        let warped = warp_perspective_bilinear(&img, &h, dst_w, dst_h)
            .map_err(|e| RasterError::Fec(format!("warp: {e:?}")))?;

        let mut params_copy = p.clone();
        params_copy.cell_px = manifest.cell_px;
        decode_frame_bytes_inner(&warped, &params_copy)
    } else {
        decode_frame_bytes(path, p)
    }
}

/// Inner decode that works on an already-loaded image.
fn decode_frame_bytes_inner(
    img: &image::ImageBuffer<Rgb<u8>, Vec<u8>>,
    p: &RasterParams,
) -> Result<Vec<u8>, RasterError> {
    let total_cells = (p.grid_w * p.grid_h) as usize;
    let mut symbols = vec![0u8; total_cells];

    // FIX: needless_range_loop — iterate with enumerate
    for (cell_idx, slot) in symbols.iter_mut().enumerate() {
        let cx = (cell_idx % p.grid_w as usize) as u32;
        let cy = (cell_idx / p.grid_w as usize) as u32;
        let px = cx * p.cell_px + p.cell_px / 2;
        let py = cy * p.cell_px + p.cell_px / 2;
        let pixel = img.get_pixel(px, py);
        *slot = p.palette.symbol_from_rgb_nearest(pixel[0], pixel[1], pixel[2]);
    }

    let payload_bits  = total_cells * 3;
    let payload_bytes = payload_bits / 8;
    let mut out = vec![0u8; payload_bytes];
    read_3bits(&symbols, &mut out);
    Ok(out)
}

/// Returns the index of the first data frame, skipping sync and calibration
/// frames. Uses the encode-time params (sync_frames + calibration_frames)
/// which are always available on the decode path, clamped to manifest.frames
/// so a corrupt/mismatched manifest can't cause an out-of-bounds read.
fn detect_data_start(manifest: &EncodeManifest, p: &RasterParams) -> u32 {
    let preamble = p.sync_frames + p.calibration_frames;
    preamble.min(manifest.frames)
}
