# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Goals

- Encode arbitrary bytes into a dense, robust video representation.
- Decode from:
  - Video file on disk (exact pixels possible with lossless codecs).
  - Live camera capture of a playing video (drop/blur tolerant).

## Increment 3d scope (implemented)

- Add a first Forward Error Correction (FEC) layer using **Reed–Solomon erasure coding**.

This is specifically to survive frame loss: erasure coding can reconstruct missing shards given enough redundancy, but it does not fix arbitrary bit errors—those should be detected (checksums) and treated as missing, which the crate docs warn about. [web:157]

## FEC model (draft)

- Data is divided into groups.
- Each group becomes N data shards + M parity shards.
- Each shard is independently checksummed.
- Shards are the unit that will map into frames in later increments.

Next: wire this into the raster encoder so frames carry shard headers.

