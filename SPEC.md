# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Goals

- Encode arbitrary files into a dense, robust video representation.
- Decode from:
  - Video file on disk (exact pixels possible with lossless codecs).
  - Live camera capture of a playing video (drop/blur tolerant).

## High-level pipeline

1. Input file or folder is packaged (optionally) as an archive.
2. Data is chunked into fixed-size source blocks.
3. Blocks are protected with an outer erasure layer (fountain/LT-style) so missing frames are OK.
4. Encoded symbols are mapped into per-frame grids of colored cells plus sync/metadata.

## Video structure

### Segment 0: Sync slate

- First N frames are a solid color slate (configurable) to let camera decoders lock exposure/focus.

### Segment 1: Calibration

- One or more frames include a color calibration strip (known palette) used to estimate color mapping.

### Segment 2: Data frames

Each data frame contains:

- 4 corner fiducials (for detection + orientation)
- timing border(s)
- metadata header cells
- payload grid

## Cell alphabet (initial)

- Start with 8-color palette (3 bits/cell) chosen for maximal separation.
- Decoder must support an "unknown" state for ambiguous samples.

## Container fields (draft)

- magic: "SLLV"
- version: u16
- profile: enum { archive, scan }
- frame_w, frame_h: u16
- grid_w, grid_h: u16
- palette_id: u16
- chunk_bytes: u32
- total_bytes: u64
- root_hash: 32 bytes (sha256)
- header_crc: u32

## Notes

This is intentionally a draft. The next increment will freeze:

- exact fiducial geometry
- palette values
- metadata encoding
- ECC parameters

