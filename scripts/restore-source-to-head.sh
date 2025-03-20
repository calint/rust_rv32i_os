#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
git reset --hard HEAD
