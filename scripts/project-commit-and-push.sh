#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
git add .
git commit -m "."
git push
