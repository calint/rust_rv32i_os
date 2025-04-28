#!/bin/sh
set -e
cd $(dirname "$0")
#set -x

cd ..
git add .
git commit -m "."
git push
