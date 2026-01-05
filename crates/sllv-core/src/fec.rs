use reed_solomon_erasure::galois_8::ReedSolomon;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
