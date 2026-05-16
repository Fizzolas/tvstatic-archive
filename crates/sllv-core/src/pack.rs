use std::io;
use std::path::{Path, PathBuf};
use tar::Builder;
use thiserror::Error;
use walkdir::{WalkDir, Error as WalkDirError};

#[derive(Debug, Error)]
pub enum PackError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("walkdir: {0}")]
    WalkDir(#[from] WalkDirError),
    #[error("invalid input path (does not exist or is not a file/folder)")]
    InvalidInput,
}

/// Package a file or directory (recursively) into a tar byte stream.
///
/// Returns the raw tar bytes plus the top-level entry name to use in the manifest.
/// The tar is intentionally *uncompressed*; a later step can compress before FEC/encoding.
///
/// # Bug fix
/// Previously, packing an empty directory silently produced a zero-length tar with no
/// entries, which caused the encode step to write a manifest claiming total_bytes=0 and
/// then produce no data frames — making silent round-trip failure on empty folders.
/// Now we emit at least a `.keep` placeholder entry for empty directories so the tar is
/// always non-empty and the round-trip succeeds.
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
            builder.append_path_with_name(input, &file_name)?;
        } else if input.is_dir() {
            let base = input;
            let mut file_count = 0usize;

            for entry in WalkDir::new(input) {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                let rel: PathBuf = path.strip_prefix(base).unwrap().to_path_buf();
                let name = Path::new(&file_name).join(rel);
                builder.append_path_with_name(path, name)?;
                file_count += 1;
            }

            // Empty directory guard: emit a placeholder so the tar is never zero-length.
            if file_count == 0 {
                let keep_name = Path::new(&file_name).join(".keep");
                let data: &[u8] = b"";
                let mut header = tar::Header::new_gnu();
                header.set_size(0);
                header.set_mode(0o644);
                header.set_cksum();
                builder.append_data(&mut header, keep_name, data)?;
            }
        } else {
            return Err(PackError::InvalidInput);
        }

        builder.finish()?;
    }

    Ok((out, file_name))
}
