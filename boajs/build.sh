#!/bin/sh

NAME="backend"
TARGET="wasm32-wasip1"
OUTPUT="target/${TARGET}/release/${NAME}.wasm"

# Build
cargo build \
    --release \
    --target "${TARGET}" \
    --target-dir target \
    -p "${NAME}" \
    --locked

# Wasi
wasi-shim \
    -f "${OUTPUT}" \
    -o "${OUTPUT}"

# Shrink
ic-wasm "${OUTPUT}" shrink

# Compress
gzip -f "${OUTPUT}"
