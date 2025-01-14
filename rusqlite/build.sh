#!/bin/sh

NAME="backend"
TARGET="wasm32-wasip1"
OUTPUT="target/${TARGET}/release/${NAME}.wasm"

# WASI Flags (for cargo build)
export WASI_SDK_PATH_DEFAULT='/opt/wasi-sdk'
export WASI_SDK_PATH=${WASI_SDK_PATH:-$WASI_SDK_PATH_DEFAULT}
export CC_wasm32_wasip1="${WASI_SDK_PATH}/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot" \

if [ ! -d "$WASI_SDK_PATH" ]; then
    echo "Error: Directory $WASI_SDK_PATH does not exist" >&2
    exit 1
fi

# Build
cargo build \
    --release \
    --target "${TARGET}" \
    --target-dir target \
    -p "${NAME}" \
    --locked

# WASI
wasi-shim \
    -f "${OUTPUT}" \
    -o "${OUTPUT}"

# Shrink
ic-wasm "${OUTPUT}" shrink

# Compress
gzip -f "${OUTPUT}"
