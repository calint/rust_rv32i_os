#!/bin/sh
set -e
cd $(dirname "$0")

scripts/firmware-build.sh

# run
echo " * build emulator"
emulator/make.sh
emulator/qa/test.sh

# run tests
qa/emulator/test.sh

echo " * run emulator"
emulator/osqa firmware.img sdcard.img
