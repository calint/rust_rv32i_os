.global _start
_start:
    la a0, __bss_start__
    la a1, __bss_end__
    li a2, 0
clear_loop:
    bge a0, a1, clear_done
    sb a2, (a0)
    addi a0, a0, 1
    j clear_loop
clear_done:
    li sp, 0x200000
    j run
