# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Goals

- Encode arbitrary bytes into a dense, robust video representation.
- Decode from:
  - Video file on disk (exact pixels possible with lossless codecs).
  - Live camera capture of a playing video (drop/blur tolerant).

## Increment 3b scope (implemented)

- Add a **sync slate** (solid color frames) at the start of every encode.
- Add a **calibration frame** after the sync slate containing a palette strip and a checkerboard.

This aligns with the overall goal of camera scanning, where exposure/white balance and palette classification matter. Color calibration/white balance tools exist specifically to stabilize color under varying lighting/cameras, and the calibration frame is an in-band analog of that idea. [web:145]

## High-level pipeline

1. Input path is packaged into a single `tar` byte stream.
2. Data is chunked into fixed-size payloads.
3. A fixed number of sync/calibration frames are prepended.
4. Each payload chunk becomes one data frame.

> Note: Fountain codes + stronger ECC are planned for later increments.

## Frame structure (Increment 3b)

- Segment 0: Sync slate (default 30 frames)
- Segment 1: Calibration (default 1 frame)
- Segment 2: Data frames

