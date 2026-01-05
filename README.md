# tvstatic-archive

A cross-platform **visual file codec** that encodes arbitrary bytes into a dense, TV-static-like color-cell video, and decodes them back.

This repo implements two profiles:

- **Archive profile**: lossless storage video (Matroska + FFV1) so frames survive exactly.
- **Scan profile**: camera-friendly output for screen-to-phone capture with stronger redundancy.

## CLI quickstart

Encode to frames + MKV (FFV1):

```bash
sllv encode --input ./my_folder --out-frames ./frames --out-mkv ./out.mkv --fps 24
```

Decode from MKV (extract frames then decode):

```bash
sllv decode --input-mkv ./out.mkv --out-tar ./recovered.tar
```

This uses ffmpeg's documented image sequence extraction and sync options. [web:173]

Decode from frames:

```bash
sllv decode --input-frames ./frames --out-tar ./recovered.tar
```

## Status

- ✅ Increment 0: repo + draft spec + Rust workspace scaffold
- ✅ Increment 1a: lossless PNG frame encoder/decoder + optional FFmpeg (FFV1/MKV) wrapper
- ✅ Increment 1b: package ANY input (file or folder) into a single archive, then encode/decode
- ✅ Increment 2a/2b/2c: desktop GUI (Tauri) usable with pickers + progress
- ✅ Increment 3a (Android): usable encode/decode via SAF staging + JNI calling Rust
- ✅ Increment 3b/3c: sync+calibration + border + nearest-palette decode
- ✅ Increment 3d/3e: RS shards + per-shard hash + auto-detect
- ✅ Increment 3f/3g: CLI encode/decode supports MKV via ffmpeg
- ⏭️ Next: fiducials + camera pipeline + GUI scan-profile presets

