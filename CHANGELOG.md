# Changelog

This changelog tracks user-visible changes.

## Unreleased

- Add desktop + Android UI profile selection.
- Improve fiducial detection robustness (L-shape verification, palette-based sampling).
- Add bilinear resampling option for deskew warps.

## 0.0.8 (2026-01-05)

- Added Scan vs Archive profile presets.
- Decode now accepts RasterParams so profiles control deskew + FEC.
- Added pure-Rust homography + warp and integrated fiducial-based deskew in decode.

## 0.0.7 (2026-01-05)

- RS (Reed-Solomon) shard framing with per-shard SHA-256.
- Added border + calibration/sync frames.
- Added colored corner fiducials.
