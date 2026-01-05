use reed_solomon_erasure::galois_8::ReedSolomon;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct FecParams {
    /// Number of data shards per group.
    pub data_shards: usize,
    /// Number of parity shards per group.
    pub parity_shards: usize,
    /// Size of each shard (bytes). Must be consistent within a group.
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

/// Encode bytes into a stream of (group_index, shard_index, shard_bytes, shard_sha256).
///
/// This is a forward-error-correction layer intended for "scan profile": if some frames are lost,
/// missing shards can be reconstructed as long as enough shards remain.
///
/// Note: This is erasure coding (missing shards), not bit-error correction. Per-frame corruption
/// should be detected (checksum) and treated as missing. docs.rs notes this explicitly. [web:157]
pub fn fec_encode_stream(data: &[u8], p: &FecParams) -> Result<Vec<ShardPacket>, FecError> {
    if p.data_shards == 0 || p.parity_shards == 0 || p.shard_bytes == 0 {
        return Err(FecError::InvalidParams);
    }

    let rs = ReedSolomon::new(p.data_shards, p.parity_shards)
        .map_err(|e| FecError::Rs(format!("{e:?}")))?;

    // Pad input to multiple of data_shards*shard_bytes.
    let group_data_bytes = p.data_shards * p.shard_bytes;
    let mut padded = data.to_vec();
    let rem = padded.len() % group_data_bytes;
    if rem != 0 {
        padded.resize(padded.len() + (group_data_bytes - rem), 0);
    }

    let mut packets = Vec::new();

    for (group_index, group) in padded.chunks(group_data_bytes).enumerate() {
        let mut shards: Vec<Vec<u8>> = Vec::with_capacity(p.data_shards + p.parity_shards);

        // Data shards
        for i in 0..p.data_shards {
            let start = i * p.shard_bytes;
            let end = start + p.shard_bytes;
            shards.push(group[start..end].to_vec());
        }

        // Parity shards placeholder
        for _ in 0..p.parity_shards {
            shards.push(vec![0u8; p.shard_bytes]);
        }

        rs.encode(&mut shards)
            .map_err(|e| FecError::Rs(format!("{e:?}")))?;

        for (shard_index, shard) in shards.into_iter().enumerate() {
            let mut hasher = Sha256::new();
            hasher.update(&shard);
            let sha = hasher.finalize();
            packets.push(ShardPacket {
                group_index: group_index as u32,
                shard_index: shard_index as u16,
                shard_bytes: shard,
                shard_sha256: sha.into(),
            });
        }
    }

    Ok(packets)
}

/// Attempt to reconstruct all data from collected shard packets.
///
/// Caller should supply all packets they managed to decode from frames.
pub fn fec_decode_collect(
    mut packets: Vec<ShardPacket>,
    original_len: usize,
    p: &FecParams,
) -> Result<Vec<u8>, FecError> {
    let rs = ReedSolomon::new(p.data_shards, p.parity_shards)
        .map_err(|e| FecError::Rs(format!("{e:?}")))?;

    // Group packets by group_index.
    packets.sort_by_key(|x| (x.group_index, x.shard_index));

    let mut out = Vec::new();

    let mut i = 0;
    while i < packets.len() {
        let g = packets[i].group_index;

        // Build shard vec<Option<Vec<u8>>> for this group.
        let mut shards: Vec<Option<Vec<u8>>> = vec![None; p.data_shards + p.parity_shards];

        while i < packets.len() && packets[i].group_index == g {
            let pkt = &packets[i];
            // Verify shard checksum; if mismatch, treat as missing.
            let mut hasher = Sha256::new();
            hasher.update(&pkt.shard_bytes);
            let sha: [u8; 32] = hasher.finalize().into();
            if sha == pkt.shard_sha256 {
                if (pkt.shard_index as usize) < shards.len() {
                    shards[pkt.shard_index as usize] = Some(pkt.shard_bytes.clone());
                }
            }
            i += 1;
        }

        rs.reconstruct(&mut shards)
            .map_err(|e| FecError::Rs(format!("{e:?}")))?;

        // Append only data shards.
        for s in shards.into_iter().take(p.data_shards) {
            let s = s.ok_or_else(|| FecError::Rs("missing shard after reconstruct".into()))?;
            out.extend_from_slice(&s);
        }
    }

    out.truncate(original_len);
    Ok(out)
}

#[derive(Debug, Clone)]
pub struct ShardPacket {
    pub group_index: u32,
    pub shard_index: u16,
    pub shard_bytes: Vec<u8>,
    pub shard_sha256: [u8; 32],
}
