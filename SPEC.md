# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Increment 3k scope (implemented)

- Decoder now accepts explicit parameters (profile presets can reliably influence deskew + FEC).

This matters because scan workflows need perspective correction (homography warp) and redundancy, while archive workflows can skip them when using a lossless FFV1 pipeline. [web:60][web:258]

