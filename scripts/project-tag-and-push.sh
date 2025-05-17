#!/bin/sh
set -e
cd $(dirname "$0")

TAG=$(date "+%Y-%m-%d--%H-%M")

cd ..
git tag $TAG
git push origin $TAG
