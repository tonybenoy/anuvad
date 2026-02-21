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

echo "==> Building extension with Trunk..."
trunk build --release \
    --config Trunk.extension.toml \
    --features extension

echo "==> Assembling extension_dist/..."
rm -rf extension_dist
cp -r dist extension_dist
cp extension/manifest.json extension_dist/
cp extension/background.js extension_dist/

echo "==> Extension build complete!"
echo "Load extension_dist/ as an unpacked extension in Chrome."
