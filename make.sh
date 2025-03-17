#!/bin/sh
set -e
cd $(dirname "$0")

BIN=target/riscv32i-unknown-none-elf/release/firmware
OBJCOPY=riscv64-elf-objcopy
OBJDUMP=riscv64-elf-objdump

cargo build --release

$OBJCOPY $BIN -O binary firmware.bin
$OBJDUMP --source-comment -Mnumeric,no-aliases -Sr $BIN > firmware.lst
#$OBJDUMP --source-comment -Sr $BIN > firmware.lst
$OBJDUMP -s --section=.rodata --section=.srodata --section=.data --section=.sdata --section=.bss --section=.sbss $BIN > firmware.dat || true

# run
echo " * build emulator"
emulator/make.sh

echo " * run emulator"
emulator/osqa firmware.bin sdcard.bin