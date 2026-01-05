# Android (Increment 3a)

This folder contains the Android app.

## Build steps (development)

1) Install Android Studio + SDK + NDK.
2) Build the Rust shared library for Android ABIs.
3) Open this folder in Android Studio and run on a device.

## Building the Rust `.so`

From repo root (requires `cargo-ndk`):

```bash
cargo install cargo-ndk
cargo ndk -t arm64-v8a -t armeabi-v7a -o apps/sllv-android/app/src/main/jniLibs build -p sllv-ffi --release
```

The Android app loads the library with `System.loadLibrary("sllv_ffi")`.

## Storage model

This app uses the Storage Access Framework to pick files/folders and write output in a user-chosen location. Android's docs recommend `ACTION_OPEN_DOCUMENT_TREE` for user-selected directory access. [web:118]

