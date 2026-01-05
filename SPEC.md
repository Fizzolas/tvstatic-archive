# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Increment 3e scope (implemented)

- Store the **full per-shard SHA-256** in the shard header.
- Add **CRC32** on the shard header to quickly reject corrupted headers.
- Add a heuristic **auto-detection** of sync+calibration so decode is less brittle.

Erasure coding relies on treating corrupted shards as missing (not as wrong data), so per-shard verification is necessary for robust scanning pipelines. [web:157]

