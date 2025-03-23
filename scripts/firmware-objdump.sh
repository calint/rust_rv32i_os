#!/bin/sh
set -e
cd $(dirname "$0")

cd ..
riscv64-elf-objdump -x target/riscv32i-unknown-none-elf/release/firmware