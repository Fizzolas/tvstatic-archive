# tvstatic-archive

A cross-platform **visual file codec** that encodes files into a dense, TV-static-like color-cell video, and decodes them back.

This repo will implement two profiles:

- **Archive profile**: lossless storage video (Matroska + FFV1) so frames survive exactly.
- **Scan profile**: camera-friendly output for screen-to-phone capture with stronger redundancy.

## Status

Scaffold + specification draft placeholders are in place. Next increment: implement the core bitstream container + a minimal encode/decode round-trip CLI.

## Build (planned)

- Core: Rust workspace
- Desktop UI: Tauri (Rust + WebView)
- Android: Kotlin + Rust via FFI

See `SPEC.md` for the on-disk/on-video format draft.
