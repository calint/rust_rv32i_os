#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
git tag $(date "+%Y-%m-%d--%H-%M")
git push origin
