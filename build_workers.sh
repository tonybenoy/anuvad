#!/bin/bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"

echo "==> Building Whisper worker WASM..."
wasm-pack build crates/anuvad-whisper \
    --target web \
    --out-dir ../../workers/pkg_whisper \
    --out-name anuvad_whisper \
    -- --features "" \
    2>&1 | tail -5

echo "==> Building Translator worker WASM..."
wasm-pack build crates/anuvad-translator \
    --target web \
    --out-dir ../../workers/pkg_translator \
    --out-name anuvad_translator \
    -- --features "" \
    2>&1 | tail -5

echo "==> Worker WASM builds complete!"
echo "Now run: trunk serve"
