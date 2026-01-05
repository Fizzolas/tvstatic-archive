# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Goals

- Encode arbitrary bytes into a dense, robust video representation.
- Decode from:
  - Video file on disk (exact pixels possible with lossless codecs).
  - Live camera capture of a playing video (drop/blur tolerant).

## Increment 3c scope (implemented)

- Add a border around every frame (black/white checker) to help future camera detection/deskew.
- Switch decode from exact RGB equality to **nearest-palette** classification to tolerate non-perfect capture.

Fiducial systems (e.g., AprilTag) exist specifically to make visual detection robust under perspective and noise, and this border is a simple first step toward that kind of robustness. [web:139][web:142]

## Frame structure (Increment 3c)

- Segment 0: Sync slate (default 30 frames)
- Segment 1: Calibration (default 1 frame)
- Segment 2: Data frames

Each frame now has:
- Outer border region (checker pattern)
- Inner payload region (colored cells)

