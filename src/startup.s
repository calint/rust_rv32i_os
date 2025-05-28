# generated - do not edit (see `configuration.py`)
.global _start
_start:
    # initialize BSS section to zeros
    la a0, __bss_start__
    la a1, __bss_end__
    li a2, 0
.bss_clear_loop:
    bge a0, a1, .bss_clear_done
    sb a2, (a0)
    addi a0, a0, 1
    j .bss_clear_loop
.bss_clear_done:
    # set stack pointer and enter program
    li sp, 0x800000
    j run
