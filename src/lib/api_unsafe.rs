use super::constants::*;
use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};

unsafe extern "C" {
    // declared in 'linker.ld
    pub unsafe static __heap_start__: u8;
}

#[inline(always)]
pub fn uart_send_char(ch: u8) {
    unsafe {
        while read_volatile(UART_OUT_ADDR as *const i32) != -1 {}
        write_volatile(UART_OUT_ADDR as *mut u8, ch);
    }
}

#[inline(always)]
pub fn uart_read_char() -> u8 {
    unsafe {
        loop {
            let input = read_volatile(UART_IN_ADDR as *const i32);
            if input != -1 {
                return input as u8;
            }
        }
    }
}

#[inline(always)]
pub fn led_set(low_being_on_bits: u8) {
    unsafe { write_volatile(LED as *mut u8, low_being_on_bits) }
}

#[inline(always)]
pub fn memory_stack_pointer() -> u32 {
    let sp: u32;
    unsafe {
        asm!(
            "mv {0}, sp",
            out(reg) sp,
        );
    }
    sp
}

#[inline(always)]
pub fn sdcard_status() -> i32 {
    unsafe { read_volatile(SDCARD_STATUS as *const i32) }
}

pub fn sdcard_read_blocking(sector: u32, buffer_512_bytes: &mut [u8; 512]) {
    unsafe {
        while read_volatile(SDCARD_BUSY as *const i32) != 0 {}
        write_volatile(SDCARD_READ_SECTOR as *mut u32, sector);
        while read_volatile(SDCARD_BUSY as *const i32) != 0 {}
        for i in 0..512 {
            buffer_512_bytes[i] = read_volatile(SDCARD_NEXT_BYTE as *const u8);
        }
    }
}

pub fn sdcard_write_blocking(sector: u32, buffer_512_bytes: &[u8; 512]) {
    unsafe {
        while read_volatile(SDCARD_BUSY as *const i32) != 0 {}
        for i in 0..512 {
            write_volatile(SDCARD_NEXT_BYTE as *mut u8, buffer_512_bytes[i]);
        }
        write_volatile(SDCARD_WRITE_SECTOR as *mut u32, sector);
        while read_volatile(SDCARD_BUSY as *const i32) != 0 {}
    }
}
