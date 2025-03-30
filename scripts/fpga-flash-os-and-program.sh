#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
scripts/firmware-build.sh
openFPGALoader --external-flash firmware.img
openFPGALoader ../tang-nano-9k--riscv--cache-psram/impl/pnr/riscv.fs
#openFPGALoader --write-flash ../tang-nano-9k--riscv--cache-psram/impl/pnr/riscv.fs
