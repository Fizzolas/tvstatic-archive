# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Goals

- Encode arbitrary bytes into a dense, robust video representation.
- Decode from:
  - Video file on disk (exact pixels possible with lossless codecs).
  - Live camera capture of a playing video (drop/blur tolerant).

## Increment 1b scope (implemented)

- Accept ANY input:
  - file => archived as tar
  - directory => archived as tar (recursive)
- Encode archive bytes into frames.
- Decode frames back into the same tar bytes (sha256 verified).

Why tar?
- Works for arbitrary binary data and preserves directory structure.
- Cross-platform with standard tooling.

## High-level pipeline

1. Input path is packaged into a single `tar` byte stream.
2. Data is chunked into fixed-size payloads.
3. Each chunk becomes one frame.
4. Each frame is a grid of colored cells representing 3-bit symbols.

> Note: Fountain codes + stronger ECC are planned for later increments.

## Frame structure (Increment 1b)

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
- `payload.tar`: (optional) not written by default; used only for debugging.
- `frame_000000.png` ... `frame_N.png`: payload frames.

