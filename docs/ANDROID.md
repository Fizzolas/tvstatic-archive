# Android (Increment 3a - scaffold)

## Goal

Provide an Android app (Kotlin) that can call the Rust core to decode/encode, with a usable GUI.

## Planned approach

- Build Rust as `.so` for Android ABIs.
- Load library from Kotlin and call a small C-ABI shim.
- Later increments may migrate to UniFFI for a more ergonomic Kotlin API.

UniFFI can be integrated with Gradle by invoking `uniffi-bindgen` during builds. [web:100]

## Rust build (planned)

We will use `cargo-ndk` to build Android libraries and place them into the expected `jniLibs` directory structure. [web:110]

Example (once Android project exists):

```bash
cargo install cargo-ndk
cargo ndk -t armeabi-v7a -t arm64-v8a -o apps/sllv-android/app/src/main/jniLibs build -p sllv-ffi --release
```

