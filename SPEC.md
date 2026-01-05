# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Increment 3h scope (implemented)

- Add simple **corner fiducials** (colored L-shapes) inside the border.

Fiducial designs like AprilTag are built to stay detectable under perspective distortion and noise; while these L-shapes are much simpler than AprilTag, they’re meant to provide orientation/corner hints for the next step (homography + deskew). [web:142]

