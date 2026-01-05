#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
mkdir -p dist

cargo build -p sllv-cli --release

cp -f target/release/sllv dist/sllv
chmod +x dist/sllv

echo "Built dist/sllv"
