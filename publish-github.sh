#!/bin/bash
set -ex

set current_branch $(git branch --show-current)
git checkout gh-pages

./publish.sh

git checkout "$current_branch"
