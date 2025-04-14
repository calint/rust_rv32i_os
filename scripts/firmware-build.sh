#!/bin/sh
set -e
cd $(dirname "$0")

ELF=target/riscv32i-unknown-none-elf/release/firmware
OBJCOPY=riscv64-elf-objcopy
OBJDUMP=riscv64-elf-objdump
FIRMWARE=firmware

cd ..

cargo clean
cargo clippy --release -- -W clippy::all -W clippy::pedantic -W clippy::nursery -W clippy::correctness -W clippy::perf \
                          -W clippy::style -W clippy::suspicious \
                          -W clippy::unwrap_used -W clippy::unseparated_literal_suffix

#                          -W clippy::restriction \
#                          -W clippy::unwrap_used -W clippy::expect_used \
#                          -A clippy::single_call_fn -A clippy::indexing_slicing -A clippy::missing_docs_in_private_items \
#                          -A clippy::implicit_return -A clippy::single_char_lifetime_names \
#                          -A clippy::min_ident_chars -A clippy::arithmetic_side_effects -A clippy::default_numeric_fallback \
#                          -A clippy::as_conversions -A clippy::panic -A clippy::multiple_unsafe_ops_per_block \
#                          -A clippy::undocumented_unsafe_blocks -A clippy::arbitrary_source_item_ordering \
#                          -A clippy::missing_trait_methods -A clippy::module_name_repetitions -A clippy::missing_assert_message \


cargo build --release
$OBJCOPY -O binary $ELF $FIRMWARE.img
#$OBJDUMP --source-comment -Mnumeric,no-aliases -Sr $ELF > firmware.lst
$OBJDUMP --source-comment -Sr $ELF > $FIRMWARE.lst
$OBJDUMP -s --section=.rodata --section=.srodata --section=.data --section=.sdata --section=.bss --section=.sbss $ELF > $FIRMWARE.dat || true
echo " * firmware built"
ls -l --color $FIRMWARE.img
