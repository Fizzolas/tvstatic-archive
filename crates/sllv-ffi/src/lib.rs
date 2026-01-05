use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::PathBuf;

/// Simple C ABI surface for Android/JNI to call into.
///
/// Increment 3a starts with minimal, stable functions.
/// Later increments will add progress callbacks and video decode entry points.

#[no_mangle]
pub extern "C" fn sllv_pack_and_encode_to_frames(
    input_path: *const c_char,
    out_dir: *const c_char,
) -> c_int {
    let res: anyhow::Result<()> = (|| {
        let input = unsafe { CStr::from_ptr(input_path) }.to_string_lossy().to_string();
        let out = unsafe { CStr::from_ptr(out_dir) }.to_string_lossy().to_string();
        let input = PathBuf::from(input);
        let out_dir = PathBuf::from(out);

        let (tar_bytes, name) = sllv_core::pack_path_to_tar_bytes(&input)?;
        let p = sllv_core::RasterParams::default();
        sllv_core::encode_bytes_to_frames_dir(&tar_bytes, &format!("{}.tar", name), &out_dir, &p)?;
        Ok(())
    })();

    match res {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

#[no_mangle]
pub extern "C" fn sllv_decode_frames_to_tar(in_dir: *const c_char, output_tar: *const c_char) -> c_int {
    let res: anyhow::Result<()> = (|| {
        let in_dir = unsafe { CStr::from_ptr(in_dir) }.to_string_lossy().to_string();
        let output_tar = unsafe { CStr::from_ptr(output_tar) }.to_string_lossy().to_string();
        let in_dir = PathBuf::from(in_dir);
        let output_tar = PathBuf::from(output_tar);

        let bytes = sllv_core::decode_frames_dir_to_bytes(&in_dir)?;
        std::fs::write(output_tar, bytes)?;
        Ok(())
    })();

    match res {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

// Simple last-error API (placeholder). Real error strings will be added next.
#[no_mangle]
pub extern "C" fn sllv_last_error_message() -> *const c_char {
    // placeholder: return empty string
    static EMPTY: &[u8] = b"\0";
    EMPTY.as_ptr() as *const c_char
}
