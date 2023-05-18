#!/bin/bash
set -ex

echo "Building WASM package..."
cargo build --release --features=gui --target=wasm32-unknown-unknown

echo "Running Github deploy script..."
set current_branch $(git branch --show-current)
git checkout gh-pages

./publish.sh

git checkout "$current_branch"

echo "Done!"
