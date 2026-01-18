#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
mkdir -p dist

# Linux/macOS: build the CLI by default.
# The GUI may require additional system dependencies (OpenGL/windowing). Build it manually if desired.

cargo build -p sllv-cli --release

# Binary name may differ by platform/toolchain; prefer the package-named binary when present.
if [[ -f target/release/sllv-cli ]]; then
  cp -f target/release/sllv-cli dist/sllv
elif [[ -f target/release/sllv ]]; then
  cp -f target/release/sllv dist/sllv
else
  echo "ERROR: Could not find CLI binary in target/release (expected sllv-cli or sllv)." >&2
  exit 1
fi

chmod +x dist/sllv

echo "Built dist/sllv"
