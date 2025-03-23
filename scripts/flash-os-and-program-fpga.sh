#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
scripts/build-firmware.sh
openFPGALoader firmware.img --external-flash
openFPGALoader ../tang-nano-9k--riscv--cache-psram/impl/pnr/riscv.fs 