use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FecParams {
    pub data_shards: usize,
    pub parity_shards: usize,
    pub shard_bytes: usize,
}

impl Default for FecParams {
    fn default() -> Self {
        Self {
            data_shards: 20,
            parity_shards: 10,
            shard_bytes: 1024,
        }
    }
}

#[derive(Debug, Error)]
pub enum FecError {
    #[error("invalid params")]
    InvalidParams,
}

#[derive(Debug, Clone)]
pub struct ShardPacket {
    pub group_index: u32,
    pub shard_index: u16,
    pub shard_bytes: Vec<u8>,
    pub shard_sha256: [u8; 32],
}

pub fn fec_encode_stream(input: &[u8], p: &FecParams) -> Result<Vec<ShardPacket>, FecError> {
    if p.data_shards == 0 || p.shard_bytes == 0 {
        return Err(FecError::InvalidParams);
    }

    // Minimal, deterministic "packetization" fallback for now.
    // This keeps the public API stable and unblocks Windows builds.
    // Proper RS parity can be reintroduced later with correct reconstruct wiring.
    let group_data_bytes = p.data_shards * p.shard_bytes;

    let mut out: Vec<ShardPacket> = Vec::new();
    let mut group_index: u32 = 0;

    for chunk in input.chunks(group_data_bytes) {
        for shard_index in 0..p.data_shards {
            let start = shard_index * p.shard_bytes;
            let end = std::cmp::min(start + p.shard_bytes, chunk.len());

            let mut shard_bytes = vec![0u8; p.shard_bytes];
            if start < chunk.len() {
                shard_bytes[..(end - start)].copy_from_slice(&chunk[start..end]);
            }

            let mut h = Sha256::new();
            h.update(&shard_bytes);
            let sha: [u8; 32] = h.finalize().into();

            out.push(ShardPacket {
                group_index,
                shard_index: shard_index as u16,
                shard_bytes,
                shard_sha256: sha,
            });
        }

        // Emit parity shards as zero-filled (placeholder)
        for parity_i in 0..p.parity_shards {
            let shard_index = p.data_shards + parity_i;
            let shard_bytes = vec![0u8; p.shard_bytes];
            let mut h = Sha256::new();
            h.update(&shard_bytes);
            let sha: [u8; 32] = h.finalize().into();

            out.push(ShardPacket {
                group_index,
                shard_index: shard_index as u16,
                shard_bytes,
                shard_sha256: sha,
            });
        }

        group_index = group_index.wrapping_add(1);
    }

    Ok(out)
}

pub fn fec_decode_collect(packets: Vec<ShardPacket>, total_bytes: usize, p: &FecParams) -> Result<Vec<u8>, FecError> {
    if p.data_shards == 0 || p.shard_bytes == 0 {
        return Err(FecError::InvalidParams);
    }

    use std::collections::BTreeMap;
    let mut by_group: BTreeMap<u32, Vec<Option<Vec<u8>>>> = BTreeMap::new();

    let total_shards = p.data_shards + p.parity_shards;
    for pkt in packets {
        let entry = by_group
            .entry(pkt.group_index)
            .or_insert_with(|| (0..total_shards).map(|_| None).collect());
        let idx = pkt.shard_index as usize;
        if idx < entry.len() {
            entry[idx] = Some(pkt.shard_bytes);
        }
    }

    let mut out: Vec<u8> = Vec::with_capacity(total_bytes);

    // Fallback decode: concatenate data shards in order.
    for (_g, shards) in by_group {
        for i in 0..p.data_shards {
            if let Some(bytes) = &shards[i] {
                out.extend_from_slice(bytes);
            }
        }
        if out.len() >= total_bytes {
            break;
        }
    }

    out.truncate(total_bytes);
    Ok(out)
}
