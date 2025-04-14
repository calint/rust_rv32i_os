SECTIONS
{
  .text : ALIGN(4) { *(.text .text.*) }
  .rodata : ALIGN(4) { *(.rodata .rodata.*) }
  .data : ALIGN(4) { *(.data .data.*) }
  .sdata : ALIGN(4) { *(.sdata .sdata.*) }
  . = ALIGN(4);
  __bss_start__ = .;
  .sbss : { *(.sbss .sbss.*) }
  .bss : { *(.bss .bss.*) }
  __bss_end__ = .;
  . = ALIGN(16);
  __heap_start__ = .;
}

/* note: .sdata and .sbss sections placed next to each other to optimize 'gp' register offsets */
