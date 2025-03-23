#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
emulator/osqa firmware.bin sdcard.bin