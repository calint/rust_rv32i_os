#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
riscv64-elf-objdump --all-headers target/riscv32i-unknown-none-elf/release/firmware
