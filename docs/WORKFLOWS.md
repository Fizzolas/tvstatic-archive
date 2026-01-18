# Workflows

SLLV supports two presets (“profiles”) that trade off efficiency vs robustness.

## Archive workflow (lossless)

Use `--profile archive` when the frames are stored and transported losslessly (e.g., PNG frames in a folder, or truly lossless video).

Typical flow:

```powershell
.\dist\sllv.exe encode -i <input> -o <frames_dir> --profile archive
.\dist\sllv.exe decode -i <frames_dir> -o recovered.tar --profile archive
```

## Scan workflow (camera/screen)

Use `--profile scan` when the frames will go through a camera/screen pipeline (rotation, perspective skew, blur, compression).

Scan enables:
- Deskew using colored corner fiducials.
- FEC (forward error correction) + per-shard integrity checks.

Typical flow:

```powershell
.\dist\sllv.exe encode -i <input> -o <frames_dir> --profile scan
.\dist\sllv.exe decode -i <frames_dir> -o recovered.tar --profile scan
```

## Important rule

Encode and decode must use the same `--profile`.
