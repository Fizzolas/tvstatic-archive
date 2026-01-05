# tvstatic-archive

A cross-platform **visual file codec** that encodes arbitrary bytes into a dense, TV-static-like color-cell video, and decodes them back.

This repo implements two profiles:

- **Archive profile**: lossless storage video (Matroska + FFV1) so frames survive exactly.
- **Scan profile**: camera-friendly output for screen-to-phone capture with stronger redundancy.

## CLI quickstart

Archive profile:

```bash
sllv encode --profile archive --input ./my_folder --out-frames ./frames --out-mkv ./out.mkv --fps 24
sllv decode --profile archive --input-mkv ./out.mkv --out-tar ./recovered.tar
```

Scan profile:

```bash
sllv encode --profile scan --input ./my_folder --out-frames ./frames_scan --fps 12
sllv decode --profile scan --input-frames ./frames_scan --out-tar ./recovered.tar
```

## Notes

- Deskew uses a four-point perspective transform idea (homography + warp), similar to how common CV pipelines implement perspective correction. [web:258][web:218]
- FFV1 in Matroska is commonly used for lossless/archival workflows. [web:60]

## Status

- ✅ Increment 0: repo + draft spec + Rust workspace scaffold
- ✅ Increment 1a: lossless PNG frame encoder/decoder + optional FFmpeg (FFV1/MKV) wrapper
- ✅ Increment 1b: package ANY input (file or folder) into a single archive, then encode/decode
- ✅ Increment 2a/2b/2c: desktop GUI (Tauri) usable with pickers + progress
- ✅ Increment 3a (Android): usable encode/decode via SAF staging + JNI calling Rust
- ✅ Increment 3b-3i: sync+calibration + border + fiducials + RS shards + deskew
- ✅ Increment 3j/3k: profile presets + decode respects profile params
- ⏭️ Next: expose profiles in desktop+android GUIs

