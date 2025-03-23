#!/bin/sh
set -e
cd $(dirname "$0")

EMULATOR=../../emulator/osqa
FIRMWARE=../../firmware.img
SDCARD=../../sdcard.img

echo " * running test for 2 seconds"
echo -e "$(cat test.in)" | timeout 2 $EMULATOR $FIRMWARE $SDCARD > test.out || true

if cmp -s test.diff test.out; then
    echo "test: PASSED"
    rm test.out
else
    echo "test: FAILED, check 'diff test.diff test.out'"
    exit 1
fi