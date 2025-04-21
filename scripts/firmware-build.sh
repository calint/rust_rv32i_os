#!/bin/sh
set -e
cd $(dirname "$0")

ELF=target/riscv32i-unknown-none-elf/release/firmware
OBJCOPY=riscv64-elf-objcopy
OBJDUMP=riscv64-elf-objdump
FIRMWARE=firmware
FIRMWARE_IMG="$FIRMWARE.img"
FIRMWARE_LIST="$FIRMWARE.lst"
FIRMWARE_DATA="$FIRMWARE.dat"
FIRMWARE_LOG="notes/firmware-size-and-changed-log.txt"
FIRMWARE_TMP="$FIRMWARE.img.tmp"

cd ..

cargo clean
cargo clippy --release -- \
  -W clippy::all -W clippy::pedantic -W clippy::nursery -W clippy::correctness -W clippy::perf -W clippy::style -W clippy::suspicious \
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

# Check if firmware.img already exists and make a backup.
if [ -f "$FIRMWARE_IMG" ]; then
  cp "$FIRMWARE_IMG" "$FIRMWARE_TMP"
else
  # first build
  touch "$FIRMWARE_IMG"
  touch "$FIRMWARE_TMP"
fi
old_file_size=$(stat -c "%s" "$FIRMWARE_IMG")

$OBJCOPY -O binary "$ELF" "$FIRMWARE_IMG"
$OBJDUMP --source-comment -SCr "$ELF" > "$FIRMWARE_LIST"
$OBJDUMP -s --section=.rodata --section=.srodata --section=.data --section=.sdata --section=.bss --section=.sbss "$ELF" > "$FIRMWARE_DATA" || true

echo " * firmware built"
ls -l --color "$FIRMWARE_IMG"

file_size=$(stat -c "%s" "$FIRMWARE_IMG")
timestamp=$(date +"%Y-%m-%d %H:%M:%S")

# Compare the old and new firmware.img files.
if cmp -s "$FIRMWARE_TMP" "$FIRMWARE_IMG"; then
  echo "$timestamp: $file_size B  (same)" >> "$FIRMWARE_LOG"
else
  if [ $file_size -eq $old_file_size ]; then
    echo "$timestamp: $file_size B  (changed)" >> "$FIRMWARE_LOG"
  else
    echo "$timestamp: $file_size B" >> "$FIRMWARE_LOG"
  fi
fi

# Clean up the temporary file.
rm "$FIRMWARE_TMP"