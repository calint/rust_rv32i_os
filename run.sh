#!/bin/sh
set -e
cd $(dirname "$0")

touch firmware.img.bak
cp -a firmware.img firmware.img.bak || true
scripts/firmware-build.sh

echo " * comparing previous firmware.img with current"
if cmp -s firmware.img firmware.img.bak; then
    echo "no change"
else
    old_size=$(stat -c%s firmware.img.bak)
    new_size=$(stat -c%s firmware.img)
    size_diff=$((new_size - old_size))
    echo "changed, new image size difference: $size_diff bytes"
fi

rm firmware.img.bak

# run
echo " * build emulator"
emulator/make.sh
emulator/qa/test.sh

# run tests
qa/emulator/test.sh

echo " * run emulator"
emulator/osqa firmware.img sdcard.img
