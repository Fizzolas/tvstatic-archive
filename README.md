# SLLV — Static Lattice Video Codec

SLLV turns files/folders into a sequence of “TV static” frames (or an optional lossless video) and can recover the original bytes later.

## Install (no coding required)

Right now there are **no prebuilt downloads** in this repo.

To use SLLV today, you download the ZIP and run one of the build scripts below. These scripts build a standalone `sllv.exe` you can run.

### Windows

1) Install Rust (this installs the `cargo` command):
   - https://www.rust-lang.org/tools/install

2) Download this repo as a ZIP and unzip it.

3) Run one of these:

- `scripts\build.bat` (Command Prompt)
- `scripts\build.ps1` (PowerShell)

When the build finishes, you will have:

- `dist\sllv.exe`

Then run:

```bat
dist\sllv.exe doctor
```

### macOS / Linux

1) Install Rust:
   - https://www.rust-lang.org/tools/install

2) In a terminal:

```bash
./scripts/build.sh
./dist/sllv doctor
```

## What it does

- Packs your input (file or folder) into a tar archive.
- Encodes bytes into color-cell frames (8-color palette, 3 bits per cell).
- Optional: wraps frames into Matroska/FFV1 for lossless archival storage (needs ffmpeg).
- Decodes frames (or MKV) back into the original tar bytes.

## Quick commands

Encode (Archive profile):

```bash
./dist/sllv encode --profile archive --input ./my_folder --out-frames ./frames --out-mkv ./out.mkv --fps 24
```

Encode (Scan profile):

```bash
./dist/sllv encode --profile scan --input ./my_folder --out-frames ./frames_scan --fps 12
```

Decode:

```bash
./dist/sllv decode --profile scan --input-frames ./frames_scan --out-tar ./recovered.tar
```

## FFmpeg (only for MKV)

FFmpeg is only needed when you use `--out-mkv` or `--input-mkv`.

If ffmpeg isn’t on PATH, pass it explicitly:

```bash
./dist/sllv encode ... --ffmpeg-path C:\\path\\to\\ffmpeg.exe
./dist/sllv decode ... --ffmpeg-path C:\\path\\to\\ffmpeg.exe
```

FFV1 in Matroska is commonly used for lossless/archival workflows. [web:60]

## Changelog

See [CHANGELOG.md](./CHANGELOG.md).
