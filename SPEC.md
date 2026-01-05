# SLLV (Static-Lattice Lossless Video) — Draft Spec

> Working name for the custom symbology/container.

## Increment 3j scope (implemented)

- Add explicit **profile presets**: Archive vs Scan.

Scan workflows need perspective correction and tolerate dropped frames; this is why the scan preset enables deskew and erasure coding, while the archive preset can disable them when using a lossless FFV1 pipeline. [web:60][web:258]

