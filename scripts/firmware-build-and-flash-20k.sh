#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
./configure.py 20k
scripts/firmware-build.sh
openFPGALoader --board tangnano20k --external-flash --offset=0x700000 firmware.img
