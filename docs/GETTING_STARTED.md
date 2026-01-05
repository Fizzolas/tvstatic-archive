# Getting started (new users)

## Choose a workflow

- **Archive workflow**: best for storage.
  - Output: a frames directory and/or a lossless MKV (Matroska + FFV1).
  - Requires FFmpeg only if you use MKV.

- **Scan workflow**: best for screen-to-phone transfer.
  - Output: a frames directory tuned for camera capture.
  - No FFmpeg required.

## Build

### Windows

- `scripts\build.bat` (cmd)
- `scripts\build.ps1` (PowerShell)

Output: `dist\sllv.exe`

### macOS / Linux

- `./scripts/build.sh`

Output: `dist/sllv`

## First run

Run:

```bash
./dist/sllv doctor
```

If you need MKV:

```bash
./dist/sllv doctor --check-ffmpeg
```

## Common commands

Archive encode + decode:

```bash
./dist/sllv encode --profile archive --input ./my_folder --out-frames ./frames --out-mkv ./out.mkv --fps 24
./dist/sllv decode --profile archive --input-mkv ./out.mkv --out-tar ./recovered.tar
```

Scan encode + decode:

```bash
./dist/sllv encode --profile scan --input ./my_folder --out-frames ./frames_scan --fps 12
./dist/sllv decode --profile scan --input-frames ./frames_scan --out-tar ./recovered.tar
```

## If ffmpeg isn't found

Either install ffmpeg so it’s on PATH, or pass `--ffmpeg-path`.

FFmpeg’s wiki documents FFV1 usage for archival encoding. [web:60]
