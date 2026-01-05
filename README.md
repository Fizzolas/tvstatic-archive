# SLLV — Static Lattice Video Codec

SLLV turns files/folders into a sequence of “TV static” frames (or an optional lossless video) and can recover the original bytes later.

## Install (Windows)

If you downloaded the ZIP, the easiest path is the Windows setup script:

- `scripts\setup-windows.ps1`

### Fastest way

1) Right-click the repo folder → “Open in Terminal”.
2) Run:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\setup-windows.ps1
```

If Rust isn’t installed yet, run with auto-install:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\setup-windows.ps1 -AutoInstall
```

This will:

- Install Rust (via winget) if missing. [web:354]
- Optionally install Visual Studio Build Tools (C++ workload) if needed. [web:361][web:370]
- Build `dist\sllv.exe`
- Run `dist\sllv.exe doctor`

## Install (macOS / Linux)

Use `./scripts/build.sh` (requires Rust installed).

## FFmpeg (only for MKV)

FFmpeg is only needed when you use `--out-mkv` or `--input-mkv`.

If ffmpeg isn’t on PATH, pass `--ffmpeg-path`.

FFV1 in Matroska is commonly used for lossless/archival workflows. [web:60]

## Changelog

See [CHANGELOG.md](./CHANGELOG.md).
