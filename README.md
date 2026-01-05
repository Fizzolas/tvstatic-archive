# tvstatic-archive

A cross-platform **visual file codec** that encodes files into a dense, TV-static-like color-cell video, and decodes them back.

This repo implements two profiles:

- **Archive profile**: lossless storage video (Matroska + FFV1) so frames survive exactly.
- **Scan profile**: camera-friendly output for screen-to-phone capture with stronger redundancy.

## Status

- ✅ Increment 0: repo + draft spec + Rust workspace scaffold
- ✅ Increment 1a: lossless PNG frame encoder/decoder + optional FFmpeg (FFV1/MKV) wrapper
- ⏭️ Next: freeze fiducials + add proper framing/ECC + Android/desktop UI

## Quick start (Increment 1a)

### Build

```bash
cargo build -p sllv-cli --release
```

### Encode a file into frames

```bash
./target/release/sllv encode --input path/to/file.bin --out-dir out_frames
```

### Decode frames back into the original file

```bash
./target/release/sllv decode --in-dir out_frames --output recovered.bin
```

### Optional: Wrap frames into a lossless MKV (FFV1)

If `ffmpeg` is installed:

```bash
./target/release/sllv make-video --in-dir out_frames --output out.mkv --fps 30
./target/release/sllv extract-frames --input out.mkv --out-dir extracted_frames --fps 30
```

See `SPEC.md` for the draft format.
