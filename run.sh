#!/bin/sh
set -e
cd $(dirname "$0")

scripts/firmware-build.sh

# run
echo " * build emulator"
emulator/make.sh
emulator/qa/qa.sh

# run tests
qa/emulate/test.sh

echo " * run emulator"
emulator/osqa firmware.img sdcard.img
