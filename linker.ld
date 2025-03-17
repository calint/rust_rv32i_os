MEMORY
{
  RAM : ORIGIN = 0x0, LENGTH = 2M
}

SECTIONS
{
  .text : { *(.text .text.*) } > RAM
  .rodata : { *(.rodata .rodata.*) } > RAM
  .data : { *(.data .data.*) } > RAM
  .bss : {
    __bss_start__ = .;
    *(.bss .bss.*) *(COMMON)
    __bss_end__ = .;
  } > RAM
}
