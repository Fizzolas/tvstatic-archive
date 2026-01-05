# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Increment 3i (part 1) scope (implemented)

- Add a pure-Rust homography + warp module.

This follows the same conceptual approach used in standard computer vision pipelines: compute a homography and apply a perspective warp (e.g., OpenCV `warpPerspective`). [web:215][web:258]

## Next

Wire this into the decoder: detect the four corner fiducials in an incoming (camera) frame, warp to canonical grid, then classify symbols.

