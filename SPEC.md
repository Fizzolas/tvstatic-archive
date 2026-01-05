# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Increment 3d (part 2) scope (implemented)

- Frames now carry **shards** instead of raw file chunks.
- Reed–Solomon erasure coding groups are encoded into data+parity shards.
- Each shard becomes one frame (after sync+calibration).

This aligns with animated/visual transfer designs where frames may be dropped; robust designs often use fountain/erasure layers so recovery does not require perfect capture. [web:40][web:44]

## Current limitations (next increment)

- Shard header currently contains only a 4-byte SHA prefix; full per-shard verification will be added next.
- Decoder currently assumes default sync/calibration settings.

