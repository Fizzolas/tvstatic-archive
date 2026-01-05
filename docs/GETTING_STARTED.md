# Getting started (new users)

## The simplest way (Windows)

1) Install Rust (this provides the `cargo` command):
   https://www.rust-lang.org/tools/install

2) Download this repo as a ZIP and unzip it.

3) Double-click:

- `scripts\build.bat`

4) After it says it built `dist\sllv.exe`, run:

- `dist\sllv.exe doctor`

If the window closes too fast, run it from Command Prompt:

- Open Command Prompt
- `cd` into the unzipped folder
- Run: `scripts\build.bat`

## If the build fails

Most failures on Windows are one of these:

- Rust isn't installed.
- `cargo` isn't on PATH (re-open Command Prompt after installing Rust).
- Missing C++ build tools (install "Visual Studio Build Tools" and select "Desktop development with C++").

## Workflows

### Archive (lossless video)

- Requires ffmpeg only if you create/decode MKV.

### Scan (phone capture)

- Does not require ffmpeg.

FFmpeg’s wiki documents FFV1 usage for archival encoding. [web:60]
