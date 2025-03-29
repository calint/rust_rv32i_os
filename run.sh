#!/bin/sh
set -e
cd $(dirname "$0")

touch firmware.img.bak
cp -a firmware.img firmware.img.bak || true
scripts/firmware-build.sh

# run
echo " * build emulator"
emulator/make.sh
emulator/qa/test.sh

# run tests
qa/emulator/test.sh

old_size=$(stat -c%s "firmware.img.bak")
new_size=$(stat -c%s "firmware.img")
size_diff=$((new_size - old_size))

echo " * stats"
echo "binary size change: $size_diff bytes"
rm firmware.img.bak

echo " * run emulator"
emulator/osqa firmware.img sdcard.img
