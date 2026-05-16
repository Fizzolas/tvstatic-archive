use reed_solomon_erasure::galois_8::ReedSolomon;
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
    #[error("reed-solomon error: {0}")]
    Rs(String),
    #[error("not enough shards to reconstruct (need at least {need}, have {have})")]
    NotEnoughShards { need: usize, have: usize },
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
    if p.data_shards + p.parity_shards > 256 {
        return Err(FecError::InvalidParams);
    }

    let rs = ReedSolomon::new(p.data_shards, p.parity_shards)
        .map_err(|e| FecError::Rs(e.to_string()))?;

    let group_data_bytes = p.data_shards * p.shard_bytes;
    let mut out: Vec<ShardPacket> = Vec::new();
    let mut group_index: u32 = 0;

    for chunk in input.chunks(group_data_bytes) {
        let mut shards: Vec<Vec<u8>> = (0..p.data_shards)
            .map(|i| {
                let start = i * p.shard_bytes;
                let end = std::cmp::min(start + p.shard_bytes, chunk.len());
                let mut s = vec![0u8; p.shard_bytes];
                if start < chunk.len() {
                    s[..(end - start)].copy_from_slice(&chunk[start..end]);
                }
                s
            })
            .collect();

        for _ in 0..p.parity_shards {
            shards.push(vec![0u8; p.shard_bytes]);
        }

        rs.encode(&mut shards).map_err(|e| FecError::Rs(e.to_string()))?;

        for (shard_index, shard_bytes) in shards.into_iter().enumerate() {
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

pub fn fec_decode_collect(
    packets: Vec<ShardPacket>,
    total_bytes: usize,
    p: &FecParams,
) -> Result<Vec<u8>, FecError> {
    if p.data_shards == 0 || p.shard_bytes == 0 {
        return Err(FecError::InvalidParams);
    }
    if p.data_shards + p.parity_shards > 256 {
        return Err(FecError::InvalidParams);
    }

    let rs = ReedSolomon::new(p.data_shards, p.parity_shards)
        .map_err(|e| FecError::Rs(e.to_string()))?;

    let total_shards = p.data_shards + p.parity_shards;

    let mut by_group: std::collections::BTreeMap<u32, Vec<Option<Vec<u8>>>> =
        std::collections::BTreeMap::new();

    for pkt in packets {
        let entry = by_group
            .entry(pkt.group_index)
            .or_insert_with(|| vec![None; total_shards]);
        let idx = pkt.shard_index as usize;
        if idx < total_shards {
            let mut h = Sha256::new();
            h.update(&pkt.shard_bytes);
            let sha: [u8; 32] = h.finalize().into();
            if sha == pkt.shard_sha256 {
                entry[idx] = Some(pkt.shard_bytes);
            }
        }
    }

    let mut out: Vec<u8> = Vec::with_capacity(total_bytes);

    for (_g, mut shards) in by_group {
        let present = shards.iter().filter(|s| s.is_some()).count();

        if present < p.data_shards {
            return Err(FecError::NotEnoughShards {
                need: p.data_shards,
                have: present,
            });
        }

        let data_complete = shards[..p.data_shards].iter().all(|s| s.is_some());
        if !data_complete {
            rs.reconstruct(&mut shards)
                .map_err(|e| FecError::Rs(e.to_string()))?;
        }

        // Concatenate data shards — iterate directly to avoid needless_range_loop
        for shard in shards.iter().take(p.data_shards) {
            match shard {
                Some(bytes) => out.extend_from_slice(bytes),
                None => {
                    return Err(FecError::NotEnoughShards {
                        need: p.data_shards,
                        have: present,
                    });
                }
            }
        }

        if out.len() >= total_bytes {
            break;
        }
    }

    out.truncate(total_bytes);
    Ok(out)
}
