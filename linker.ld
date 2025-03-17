MEMORY
{
  RAM : ORIGIN = 0x0, LENGTH = 2M
}

SECTIONS
{
  .text : { *(.text .text.*) } > RAM
  .rodata : { *(.rodata .rodata.*) } > RAM
  .data : { *(.data .data.*) } > RAM
  .bss : { *(.bss .bss.*) *(COMMON) } > RAM
}
