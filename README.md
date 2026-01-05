# tvstatic-archive

A cross-platform **visual file codec** that encodes arbitrary bytes into a dense, TV-static-like color-cell video, and decodes them back.

This repo implements two profiles:

- **Archive profile**: lossless storage video (Matroska + FFV1) so frames survive exactly.
- **Scan profile**: camera-friendly output for screen-to-phone capture with stronger redundancy.

## Status

- ✅ Increment 0: repo + draft spec + Rust workspace scaffold
- ✅ Increment 1a: lossless PNG frame encoder/decoder + optional FFmpeg (FFV1/MKV) wrapper
- ✅ Increment 1b: package ANY input (file or folder) into a single archive, then encode/decode
- ✅ Increment 2a/2b/2c: desktop GUI (Tauri) usable with pickers + progress
- ✅ Increment 3a (part 2): Android app skeleton with SAF pickers (copy/FFI wiring next)
- ⏭️ Next: implement SAF URI ↔ local cache copying and call the Rust FFI; then add sync/calibration/fiducials

