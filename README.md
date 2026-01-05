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
- ✅ Increment 3a (Android): usable encode/decode via SAF staging + JNI calling Rust
- ✅ Increment 3b: sync slate + calibration frame prepended to every encode
- ⏭️ Next: fiducials/borders + nearest-color classification; then video-first workflows

