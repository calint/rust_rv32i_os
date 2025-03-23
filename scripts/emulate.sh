#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
emulator/osqa firmware.img sdcard.img