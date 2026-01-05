# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Increment 3i (part 2) scope (implemented)

- Decoder now optionally **deskews** frames using the four colored fiducials.
- Deskew pipeline: find corner fiducials → compute homography → warp to canonical grid → classify symbols.

This is the standard four-point perspective transform concept used by common CV pipelines (e.g., OpenCV). [web:258][web:218]

