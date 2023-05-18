#!/bin/bash
set -ex

echo "Make sure to run this from <master>/publish-github.sh"
cp ./target/wasm32-unknown-unknown/release/little-chemistry.wasm ./docs

echo "Publishing to GitHub..."

git add docs/*
git commit -m "Publish new version of Little Chemistry"
git push
