SECTIONS
{
  .text : { *(.text .text.*) }
  .rodata : { *(.rodata .rodata.*) }
  .data : { *(.data .data.*) }
  .bss : {
    __bss_start__ = .;
    *(.bss .bss.*) *(COMMON)
    __bss_end__ = .;
  }
}
