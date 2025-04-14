SECTIONS
{
  .text : { *(.text .text.*) }
  .rodata : { *(.rodata .rodata.*) }
  .data : { *(.data .data.*) }
  .srodata : { *(.srodata .srodata.*) }
  .sdata : { *(.sdata .sdata.*) }
  .sbss : { 
    __bss_start__ = .;
    *(.sbss .sbss.*) 
  }
  .bss : {
    *(.bss .bss.*)
    __bss_end__ = .;
  }
  . = ALIGN(16);
  __heap_start__ = .;
}

/* note: .srodata, .sdata and .sbss sections placed next to each other to optimize 'gp' register offsets */
/* note: align 4 is the default alignment for .text, .rodata, .data, .sbss, .bss */
