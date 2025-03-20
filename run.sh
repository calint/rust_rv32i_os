#!/bin/sh
set -e
cd $(dirname "$0")

scripts/build-firmware.sh

# run
echo " * build emulator"
emulator/make.sh

# run tests
qa/emulate/test.sh

echo " * run emulator"
emulator/osqa firmware.bin sdcard.bin