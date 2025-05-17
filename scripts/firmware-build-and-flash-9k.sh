#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
./configure.py 9k
scripts/firmware-build.sh
openFPGALoader --board tangnano9k --external-flash firmware.img 
