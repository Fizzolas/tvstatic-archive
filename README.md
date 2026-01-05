# SLLV — Static Lattice Video Codec

SLLV is a small tool that turns files/folders into a sequence of “TV static” frames (or a lossless video) and can recover the original bytes later.

It supports two workflows:

- **Archive**: produce frames and optionally wrap them in Matroska/FFV1 for truly lossless storage.
- **Scan**: produce camera-friendly frames (bigger cells + redundancy) meant for screen-to-phone capture.

## Build and run

### One-command build

- Windows (PowerShell):

```powershell
./scripts/build.ps1
```

- Windows (cmd):

```bat
scripts\build.bat
```

- macOS / Linux:

```bash
./scripts/build.sh
```

Binaries will be placed in `dist/`.

### Quick sanity check

```bash
./dist/sllv doctor
# If you plan to use MKV/FFV1:
./dist/sllv doctor --check-ffmpeg
```

### CLI usage

Encode:

```bash
# archive profile
./dist/sllv encode --profile archive --input ./my_folder --out-frames ./frames --out-mkv ./out.mkv --fps 24

# scan profile
./dist/sllv encode --profile scan --input ./my_folder --out-frames ./frames_scan --fps 12
```

Decode:

```bash
# decode from mkv
./dist/sllv decode --profile archive --input-mkv ./out.mkv --out-tar ./recovered.tar

# decode from frames dir
./dist/sllv decode --profile scan --input-frames ./frames_scan --out-tar ./recovered.tar
```

### FFmpeg notes

If ffmpeg isn't on PATH, pass it explicitly:

```bash
./dist/sllv encode ... --ffmpeg-path /path/to/ffmpeg
./dist/sllv decode ... --ffmpeg-path /path/to/ffmpeg
```

FFV1 in Matroska is commonly used for lossless/archival workflows. [web:60]

## Installers / packages

This repo includes a simple local build script that produces a standalone CLI binary per platform.

Optional packaging helpers are provided:

- Windows MSI: use `cargo-wix` (requires WiX Toolset installed). [web:303]
- Cross-platform release automation: `cargo-dist` (recommended for CI). [web:301]

## Changelog

See [CHANGELOG.md](./CHANGELOG.md).
