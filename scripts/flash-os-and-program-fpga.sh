#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
openFPGALoader firmware.bin --external-flash
openFPGALoader ../tang-nano-9k--riscv--cache-psram/impl/pnr/riscv.fs 