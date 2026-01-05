# SLLV (Static-Lattice Lossless Video) — Draft Spec

SLLV is a byte-to-frames codec designed around two modes:

- Archive: exact frames (optionally wrapped in Matroska/FFV1)
- Scan: camera capture with redundancy and deskew

This spec is intentionally short and focused; detailed history lives in the changelog.

## Overview

- Input is packed into a tar archive.
- Bytes are split into RS-protected shards (scan profile) and placed into frames.
- Each frame is a 2D grid of 3-bit symbols mapped to an 8-color palette.
- Optional deskew uses four corner fiducials and a homography warp (four-point perspective transform concept). [web:258][web:218]

## Profiles

### Archive

- Intended for storage: frames and/or a lossless FFV1/MKV wrapper.
- No deskew and no FEC required when the video pipeline is truly lossless. [web:60]

### Scan

- Intended for phone scanning from a display.
- Larger cells, colored corner fiducials, deskew enabled.
- RS erasure coding + per-shard SHA-256 so bad frames/shards can be dropped and still recovered.

