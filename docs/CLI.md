# CLI reference

`dist\\sllv.exe` is both a standard CLI and a double‑click friendly interactive app.

- When launched with no arguments (double-click), it opens an interactive menu.
- When launched with arguments, it behaves like a normal CLI.

> Note: `decode` always outputs a `.tar` file. Extract it with `tar -xf recovered.tar -C out_dir`.

## Commands

### `encode`

Encode a file or folder into a directory of PNG frames (and optionally an MKV).

Syntax:

```text
sllv encode -i <PATH> -o <DIR> [--out-mkv <FILE>] [--fps <N>] [--profile <archive|scan>] [--ffmpeg-path <PATH>]
```

Required:
- `-i, --input <PATH>`: input file or folder.
- `-o, --out-frames <DIR>`: output directory for `frame_000000.png`, `manifest.json`, etc.

Optional:
- `--out-mkv <FILE>`: also create a lossless MKV (FFV1 in Matroska) via ffmpeg.
- `--fps <N>`: fps for the MKV (ignored unless `--out-mkv` is used).
- `--profile <archive|scan>`: profile preset.
- `--ffmpeg-path <PATH>`: use a specific ffmpeg executable.

Examples:

```powershell
.\\dist\\sllv.exe encode -i .\\my_folder -o .\\frames_archive --profile archive
.\\dist\\sllv.exe encode -i .\\my_folder -o .\\frames_scan --profile scan
.\\dist\\sllv.exe encode -i .\\my_folder -o .\\frames --out-mkv out.mkv --fps 24 --profile archive
```

### `decode`

Decode frames or an MKV back into the original bytes, written as a `.tar` file.

Syntax:

```text
sllv decode (-i <DIR> | -m <FILE>) -o <FILE> [--profile <archive|scan>] [--ffmpeg-path <PATH>]
```

Required:
- One input source:
  - `-i, --input-frames <DIR>`: frames directory.
  - `-m, --input-mkv <FILE>`: MKV path (frames extracted to a temp directory).
- `-o, --out-tar <FILE>`: output tar file.

Examples:

```powershell
.\\dist\\sllv.exe decode -i .\\frames_archive -o recovered.tar --profile archive
.\\dist\\sllv.exe decode -m input.mkv -o recovered.tar --profile archive
```

Extract:

```powershell
mkdir out_dir
tar -xf recovered.tar -C out_dir
```

### `doctor`

Print diagnostics and optionally validate ffmpeg.

Syntax:

```text
sllv doctor [--check-ffmpeg] [--ffmpeg-path <PATH>]
```

Examples:

```powershell
.\\dist\\sllv.exe doctor
.\\dist\\sllv.exe doctor --check-ffmpeg
```
