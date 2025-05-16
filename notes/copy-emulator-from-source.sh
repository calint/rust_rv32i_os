#!/bin/sh
set -e
cd $(dirname "$0")

echo copy emulator files
T9KPTH=../../tang-nano-9k--riscv--cache-psram
PTH=..

rm -rf $PTH/emulator
cp -ra $T9KPTH/emulator/ $PTH/
rm $PTH/emulator/osqa