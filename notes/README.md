## how to record input given to emulator
`script -I log -c scripts/emulator-run.sh`

## full clippy
```
cargo clippy --fix --release -- -W clippy::all -W clippy::pedantic -W clippy::correctness -W clippy::perf -W clippy::style -W clippy::suspicious -W clippy::unwrap_used -W clippy::unseparated_literal_suffix

#    -W clippy::nursery -W clippy::restriction \
#    -W clippy::unwrap_used -W clippy::expect_used \
#    -A clippy::single_call_fn -A clippy::indexing_slicing -A clippy::missing_docs_in_private_items \
#    -A clippy::implicit_return -A clippy::single_char_lifetime_names \
#    -A clippy::min_ident_chars -A clippy::arithmetic_side_effects -A clippy::default_numeric_fallback \
#    -A clippy::as_conversions -A clippy::panic -A clippy::multiple_unsafe_ops_per_block \
#    -A clippy::undocumented_unsafe_blocks -A clippy::arbitrary_source_item_ordering \
```



## escaped characters and \0
Use `cutecom` and send files to validate FPGA function.
