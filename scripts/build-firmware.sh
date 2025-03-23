#!/bin/sh
set -e
cd $(dirname "$0")

ELF=target/riscv32i-unknown-none-elf/release/firmware
OBJCOPY=riscv64-elf-objcopy
OBJDUMP=riscv64-elf-objdump
FIRMWARE=firmware

cd ..

cargo build --release
$OBJCOPY -O binary $ELF $FIRMWARE.img
#$OBJDUMP --source-comment -Mnumeric,no-aliases -Sr $ELF > firmware.lst
$OBJDUMP --source-comment -Sr $ELF > $FIRMWARE.lst
$OBJDUMP -s --section=.rodata --section=.srodata --section=.data --section=.sdata --section=.bss --section=.sbss $ELF > $FIRMWARE.dat || true
echo " * firmware built"
ls -l --color $FIRMWARE.img
