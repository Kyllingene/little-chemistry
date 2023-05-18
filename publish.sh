#!/bin/bash
set -ex

echo "Building WASM package..."
cargo build --release --features=gui --target=wasm32-unknown-unknown

cp ./target/wasm32-unknown-unknown/release/little-chemistry.wasm ./docs

echo "Publishing to GitHub..."

git add docs/*
git commit -m "Publish new version of Little Chemistry"
git push
