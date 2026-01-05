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
    #[error("reed-solomon: {0}")]
    Rs(String),
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
    let total_shards = p.data_shards + p.parity_shards;
    let rs = ReedSolomon::new(p.data_shards, p.parity_shards).map_err(|e| FecError::Rs(e.to_string()))?;

    let group_data_bytes = p.data_shards * p.shard_bytes;
    let mut group_index: u32 = 0;
    let mut out: Vec<ShardPacket> = Vec::new();

    for chunk in input.chunks(group_data_bytes) {
        let mut shards: Vec<Vec<u8>> = Vec::with_capacity(total_shards);

        for i in 0..p.data_shards {
            let start = i * p.shard_bytes;
            let end = std::cmp::min(start + p.shard_bytes, chunk.len());
            let mut shard = vec![0u8; p.shard_bytes];
            if start < chunk.len() {
                shard[..(end - start)].copy_from_slice(&chunk[start..end]);
            }
            shards.push(shard);
        }

        for _ in 0..p.parity_shards {
            shards.push(vec![0u8; p.shard_bytes]);
        }

        let mut refs: Vec<_> = shards.iter_mut().map(|v| v.as_mut_slice()).collect();
        rs.encode(&mut refs).map_err(|e| FecError::Rs(e.to_string()))?;

        for (si, shard_bytes) in shards.into_iter().enumerate() {
            let mut h = Sha256::new();
            h.update(&shard_bytes);
            let sha: [u8; 32] = h.finalize().into();
            out.push(ShardPacket {
                group_index,
                shard_index: si as u16,
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

    let total_shards = p.data_shards + p.parity_shards;
    let rs = ReedSolomon::new(p.data_shards, p.parity_shards).map_err(|e| FecError::Rs(e.to_string()))?;

    use std::collections::BTreeMap;
    let mut by_group: BTreeMap<u32, Vec<Option<Vec<u8>>>> = BTreeMap::new();

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

    for (_g, shards_opt) in by_group {
        // Allocate buffers for all shards; mark which were originally present.
        let mut owned: Vec<Vec<u8>> = Vec::with_capacity(total_shards);
        let mut present: Vec<bool> = Vec::with_capacity(total_shards);

        for opt in shards_opt.into_iter() {
            match opt {
                Some(mut v) => {
                    if v.len() != p.shard_bytes {
                        v.resize(p.shard_bytes, 0);
                    }
                    owned.push(v);
                    present.push(true);
                }
                None => {
                    owned.push(vec![0u8; p.shard_bytes]);
                    present.push(false);
                }
            }
        }

        // Build refs with stable addresses into `owned`.
        let mut slice_refs: Vec<Option<&mut [u8]>> = owned.iter_mut().map(|v| Some(v.as_mut_slice())).collect();
        for (i, was_present) in present.iter().enumerate() {
            if !*was_present {
                slice_refs[i] = None;
            }
        }

        rs.reconstruct(&mut slice_refs).map_err(|e| FecError::Rs(e.to_string()))?;

        for i in 0..p.data_shards {
            out.extend_from_slice(&owned[i]);
        }

        if out.len() >= total_bytes {
            break;
        }
    }

    out.truncate(total_bytes);
    Ok(out)
}
