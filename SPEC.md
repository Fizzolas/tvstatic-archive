# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Goals

- Encode arbitrary files into a dense, robust video representation.
- Decode from:
  - Video file on disk (exact pixels possible with lossless codecs).
  - Live camera capture of a playing video (drop/blur tolerant).

## Increment 1a scope (implemented)

- Deterministic file → numbered PNG frames → file round-trip.
- Optional FFmpeg wrapper to create a lossless Matroska/FFV1 video from frames.

## High-level pipeline

1. Input file is read as bytes.
2. Data is chunked into fixed-size payloads.
3. Each chunk becomes one frame.
4. Each frame is a grid of colored cells representing 3-bit symbols.

> Note: Fountain codes + stronger ECC are planned for later increments.

## Frame structure (Increment 1a)

- The output image is a square lattice of cells.
- The payload area is currently the full grid (fiducials/borders will be added next).

### Parameters

- `grid_w`, `grid_h`: number of data cells.
- `cell_px`: pixel size of each cell.
- `palette`: 8 fixed RGB colors.
- `payload_bits_per_cell`: 3

## Container fields (frame directory)

The encoder writes:

- `manifest.json`: format metadata + chunk sizes + total bytes + sha256.
- `frame_000000.png` ... `frame_N.png`: payload frames.

