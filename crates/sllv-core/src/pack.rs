use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tar::Builder;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum PackError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("invalid input path")]
    InvalidInput,
}

/// Package a file or directory (recursively) into a tar byte stream.
///
/// This is deliberately *not compressed* here; later increments may optionally
/// compress before ECC/encoding.
pub fn pack_path_to_tar_bytes(input: &Path) -> Result<(Vec<u8>, String), PackError> {
    if !input.exists() {
        return Err(PackError::InvalidInput);
    }

    let file_name = input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("input")
        .to_string();

    let mut out: Vec<u8> = Vec::new();
    {
        let mut builder = Builder::new(&mut out);
        if input.is_file() {
            // Store as a single entry named after the file.
            builder.append_path_with_name(input, &file_name)?;
        } else if input.is_dir() {
            let base = input;
            for entry in WalkDir::new(input) {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                let rel: PathBuf = path.strip_prefix(base).unwrap().to_path_buf();
                let name = Path::new(&file_name).join(rel);
                builder.append_path_with_name(path, name)?;
            }
        } else {
            return Err(PackError::InvalidInput);
        }
        builder.finish()?;
    }

    // Ensure tar is finalized by dropping builder.
    Ok((out, file_name))
}
