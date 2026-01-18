# SLLV — Static Lattice Video Codec

SLLV turns a file or folder into a sequence of “TV static” frames (PNG images), and can later recover the original bytes from those frames.

On Windows, `dist\sllv.exe` is intentionally double‑click friendly: launching it with **no arguments** opens an interactive menu (encode/decode/doctor).

## What you get

After building, `dist/` contains:

- `dist\sllv.exe` — CLI + interactive menu (double-click friendly)
- `dist\sllv-gui.exe` — GUI app

## Windows quickstart (recommended)

1) Open PowerShell in the repo folder.
2) Run setup (auto-installs Rust + Build Tools if needed):

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\setup-windows.ps1 -AutoInstall
```

3) Launch:
- Double-click `dist\sllv-gui.exe` for the GUI, or
- Double-click `dist\sllv.exe` for the interactive CLI menu, or
- Run CLI commands from PowerShell (below).

## CLI commands

Run `sllv.exe --help` for the full help text.

### Encode (file/folder -> frames)

```powershell
.\dist\sllv.exe encode -i "C:\path\to\input" -o "C:\path\to\frames" --profile archive
```

Optionally also write a lossless MKV (requires ffmpeg):

```powershell
.\dist\sllv.exe encode -i "C:\path\to\input" -o "C:\path\to\frames" --out-mkv "C:\path\to\out.mkv" --fps 24 --profile archive
```

### Decode (frames/mkv -> recovered .tar)

From frames:

```powershell
.\dist\sllv.exe decode -i "C:\path\to\frames" -o "C:\path\to\recovered.tar" --profile archive
```

From mkv:

```powershell
.\dist\sllv.exe decode -m "C:\path\to\in.mkv" -o "C:\path\to\recovered.tar" --profile archive
```

Then extract the tar:

```powershell
mkdir out_dir
tar -xf "C:\path\to\recovered.tar" -C out_dir
```

### Doctor

```powershell
.\dist\sllv.exe doctor
.\dist\sllv.exe doctor --check-ffmpeg
```

## Profiles: archive vs scan

- `archive`: for exact pixels / lossless workflows (PNG frames, truly lossless video).
- `scan`: for camera/screen workflows (deskew + FEC).

Encode and decode must use the same `--profile`.

## More docs

- `docs/CLI.md` — full command reference
- `docs/WORKFLOWS.md` — archive vs scan workflows
- `docs/TROUBLESHOOTING.md` — common errors and fixes

## Changelog

See [CHANGELOG.md](./CHANGELOG.md).
