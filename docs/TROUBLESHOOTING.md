# Troubleshooting

## PowerShell: `cdC:\...` does not work

In PowerShell, `cd` and the path must be separated by a space:

```powershell
cd C:\Users\You\Downloads\tvstatic-archive\dist
```

## “unexpected argument '--input' found”

`decode` does not use `--input`.

Use either:
- `--input-frames` / `-i` (frames directory), or
- `--input-mkv` / `-m` (mkv path)

## “must provide --input-frames or --input-mkv”

`decode` requires an input source. Examples:

```powershell
.\dist\sllv.exe decode -i .\frames -o recovered.tar
.\dist\sllv.exe decode -m input.mkv -o recovered.tar
```

## “sha256 mismatch”

This means the recovered bytes did not match the manifest hash.

Common causes:
- The wrong `--profile` was used for decode (must match encode).
- The frames/video are not lossless (use `--profile scan` for camera/screen workflows).

## Build failures on Windows

If you see linker errors or `cl.exe` is missing, install Visual Studio Build Tools:
- Visual Studio 2022 Build Tools
- Workload: “Desktop development with C++”

Then re-run `scripts\setup-windows.ps1 -AutoInstall`.

## ffmpeg not found

If you use `--out-mkv` or decode from `--input-mkv`, install ffmpeg or pass `--ffmpeg-path`.

Example:

```powershell
.\dist\sllv.exe doctor --check-ffmpeg
```
