# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Increment 3g scope (implemented)

- CLI can now decode directly from a MKV by extracting a numbered PNG frame sequence with FFmpeg.

FFmpeg documents options like `-vsync` for controlling frame duplication/dropping during extraction, which matters when treating video frames as discrete packets. [web:173]

