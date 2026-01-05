# SLLV — Static Lattice Video Codec

SLLV turns files/folders into a sequence of “TV static” frames (or an optional lossless video) and can recover the original bytes later.

## What it does

- Packs your input (file or folder) into a tar archive.
- Encodes bytes into color-cell frames (8-color palette, 3 bits per cell).
- Optionally wraps frames into Matroska/FFV1 for truly lossless archival storage.
- Can decode from frames (scan workflow) or from MKV (archive workflow) back into the original tar bytes.

## Quick start

### 1) Build a standalone binary

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

The binary will be placed at:

- Windows: `dist\sllv.exe`
- macOS/Linux: `dist/sllv`

### 2) Run a sanity check

```bash
./dist/sllv doctor
```

If you plan to use MKV/FFV1:

```bash
./dist/sllv doctor --check-ffmpeg
```

## Usage

### Encode

Archive (frames + optional MKV):

```bash
./dist/sllv encode --profile archive --input ./my_folder --out-frames ./frames --out-mkv ./out.mkv --fps 24
```

Scan (frames tuned for phone capture):

```bash
./dist/sllv encode --profile scan --input ./my_folder --out-frames ./frames_scan --fps 12
```

### Decode

Decode from MKV (Archive workflow):

```bash
./dist/sllv decode --profile archive --input-mkv ./out.mkv --out-tar ./recovered.tar
```

Decode from frames directory (Scan workflow):

```bash
./dist/sllv decode --profile scan --input-frames ./frames_scan --out-tar ./recovered.tar
```

## Install files (what exists today)

This repo currently ships **build scripts**, not installers.

- `scripts/build.bat` — Windows cmd build script → produces `dist\sllv.exe`
- `scripts/build.ps1` — Windows PowerShell build script → produces `dist\sllv.exe`
- `scripts/build.sh` — macOS/Linux build script → produces `dist/sllv`

These scripts require the Rust toolchain to be installed.

### FFmpeg requirement (only for MKV)

FFmpeg is only needed when:

- You pass `--out-mkv` to `encode`, or
- You pass `--input-mkv` to `decode`.

If ffmpeg isn’t on PATH, pass it explicitly:

```bash
./dist/sllv encode ... --ffmpeg-path /path/to/ffmpeg
./dist/sllv decode ... --ffmpeg-path /path/to/ffmpeg
```

FFV1 in Matroska is commonly used for lossless/archival workflows. [web:60]

## Releases later

When a standalone binary is ready to share, GitHub Releases can host downloadable build artifacts (attach binaries on the release page). [web:333][web:319]

## Changelog

See [CHANGELOG.md](./CHANGELOG.md).
