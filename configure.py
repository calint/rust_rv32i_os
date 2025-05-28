#!/bin/python3
# generates configuration files for Verilog source, 'os', 'emulator' and clock constraints
import os
import sys

if len(sys.argv) < 2 or (sys.argv[1] != "9k" and sys.argv[1] != "20k"):
    print(" first argument must be '9k' or '20k' for Tang Nano 9K or Tang Nano 20K")
    sys.exit(1)

script_dir = os.path.dirname(os.path.realpath(__file__))
os.chdir(script_dir)

card = sys.argv[1]

with open("src/startup.s", "w") as file:
    file.write("# generated - do not edit (see `configuration.py`)\n")
    file.write(".global _start\n")
    file.write("_start:\n")
    file.write("    # initialize BSS section to zeros\n")
    file.write("    la a0, __bss_start__\n")
    file.write("    la a1, __bss_end__\n")
    file.write("    li a2, 0\n")
    file.write(".bss_clear_loop:\n")
    file.write("    bge a0, a1, .bss_clear_done\n")
    file.write("    sb a2, (a0)\n")
    file.write("    addi a0, a0, 1\n")
    file.write("    j .bss_clear_loop\n")
    file.write(".bss_clear_done:\n")
    file.write("    # set stack pointer and enter program\n")
    file.write("    li sp, 0x800000\n")
    file.write("    j run\n")

with open("src/lib/constants.rs", "w") as file:
    file.write("// generated - do not edit (see `configuration.py`)\n")
    file.write("pub const LED: u32 = 0xffff_fffc;\n")
    file.write("pub const UART_OUT_ADDR: u32 = 0xffff_fff8;\n")
    file.write("pub const UART_IN_ADDR: u32 = 0xffff_fff4;\n")
    file.write("pub const SDCARD_BUSY: u32 = 0xffff_fff0;\n")
    file.write("pub const SDCARD_READ_SECTOR: u32 = 0xffff_ffec;\n")
    file.write("pub const SDCARD_NEXT_BYTE: u32 = 0xffff_ffe8;\n")
    file.write("pub const SDCARD_STATUS: u32 = 0xffff_ffe4;\n")
    file.write("pub const SDCARD_WRITE_SECTOR: u32 = 0xffff_ffe0;\n")
    file.write("pub const MEMORY_END: u32 = 0x0080_0000;\n")

with open("emulator/src/main_config.hpp", "w") as file:
    file.write("// generated - do not edit (see `configuration.py`)\n")
    file.write("#pragma once\n")
    file.write("#include <cstdint>\n")
    file.write("\n")
    file.write("namespace osqa {\n")
    file.write("\n")
    file.write("// memory map\n")
    file.write("std::uint32_t constexpr led = 0xffff'fffc;\n")
    file.write("std::uint32_t constexpr uart_out = 0xffff'fff8;\n")
    file.write("std::uint32_t constexpr uart_in = 0xffff'fff4;\n")
    file.write("std::uint32_t constexpr sdcard_busy = 0xffff'fff0;\n")
    file.write("std::uint32_t constexpr sdcard_read_sector = 0xffff'ffec;\n")
    file.write("std::uint32_t constexpr sdcard_next_byte = 0xffff'ffe8;\n")
    file.write("std::uint32_t constexpr sdcard_status = 0xffff'ffe4;\n")
    file.write("std::uint32_t constexpr sdcard_write_sector = 0xffff'ffe0;\n")
    file.write("std::uint32_t constexpr io_addresses_start = 0xffff'ffe0;\n")
    file.write("std::uint32_t constexpr memory_end = 0x0080'0000;\n")
    file.write("\n")
    file.write("} // namespace osqa\n")
