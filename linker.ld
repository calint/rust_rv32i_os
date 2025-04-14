SECTIONS
{
  .text : { *(.text .text.*) }
  .rodata : { *(.rodata .rodata.*) }
  .data : { *(.data .data.*) }
  .sdata : { *(.sdata .sdata.*) }
  __bss_start__ = .;
  .sbss : { *(.sbss .sbss.*) }
  .bss : { *(.bss .bss.*) }
  __bss_end__ = .;
  . = ALIGN(4);
  __heap_start__ = .;
}

/* note: .sdata and .sbss sections placed next to each other to optimize 'gp' register offsets */
