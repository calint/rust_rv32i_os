//
// reviewed: 2025-04-21
//
pub(super) const MEMORY_END: u32 = 0x0020_0000;
pub(super) const LED: u32 = 0xffff_fffc;
pub(super) const UART_OUT_ADDR: u32 = 0xffff_fff8;
pub(super) const UART_IN_ADDR: u32 = 0xffff_fff4;
pub(super) const SDCARD_BUSY: u32 = 0xffff_fff0;
pub(super) const SDCARD_READ_SECTOR: u32 = 0xffff_ffec;
pub(super) const SDCARD_NEXT_BYTE: u32 = 0xffff_ffe8;
pub(super) const SDCARD_STATUS: u32 = 0xffff_ffe4;
pub(super) const SDCARD_WRITE_SECTOR: u32 = 0xffff_ffe0;
