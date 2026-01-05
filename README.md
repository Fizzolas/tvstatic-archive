# SLLV — Static Lattice Video Codec

SLLV turns files/folders into a sequence of “TV static” frames and can recover the original bytes later.

## Windows: you will get two apps

After setup/build, you will have:

- `dist\sllv.exe` — CLI (full control)
- `dist\sllv-gui.exe` — GUI (double-click friendly)

## Windows setup (ZIP download)

Run this (PowerShell):

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\setup-windows.ps1 -AutoInstall
```

Then double-click `dist\sllv-gui.exe`.

## Notes

The GUI exposes only settings that are safe (they won't silently corrupt interpretation), but anything that must match between encode/decode is grouped under “Safe settings (keep consistent for decode)”.

## Changelog

See [CHANGELOG.md](./CHANGELOG.md).
