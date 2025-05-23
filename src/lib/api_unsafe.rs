//
// reviewed: 2025-04-21
//
use super::constants::{
    LED, SDCARD_BUSY, SDCARD_NEXT_BYTE, SDCARD_READ_SECTOR, SDCARD_STATUS, SDCARD_WRITE_SECTOR,
    UART_IN_ADDR, UART_OUT_ADDR,
};
use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};

pub const SDCARD_SECTOR_SIZE_BYTES: usize = 512;

unsafe extern "C" {
    pub unsafe static __heap_start__: u8;
    // note: declared in `linker.ld`
}

pub fn uart_send_byte(byte: u8) {
    unsafe {
        while read_volatile(UART_OUT_ADDR as *const i32) != -1 {}
        write_volatile(UART_OUT_ADDR as *mut u8, byte);
    }
}

#[expect(clippy::cast_possible_truncation, reason = "intended behavior")]
#[expect(clippy::cast_sign_loss, reason = "intended behavior")]
pub fn uart_read_byte() -> u8 {
    unsafe {
        loop {
            let input = read_volatile(UART_IN_ADDR as *const i32);
            if input != -1 {
                return input as u8;
            }
        }
    }
}

pub fn led_set(bits_low_being_on: u32) {
    unsafe { write_volatile(LED as *mut u32, bits_low_being_on) }
}

pub fn memory_stack_pointer() -> u32 {
    let sp: u32;
    unsafe {
        asm!(
            "mv {0}, sp",
            out(reg) sp,
        );
    };
    sp
}

pub fn sdcard_status() -> i32 {
    unsafe { read_volatile(SDCARD_STATUS as *const i32) }
}

pub fn sdcard_read_blocking(sector: u32, buffer_512_bytes: &mut [u8]) {
    assert!(
        buffer_512_bytes.len() == SDCARD_SECTOR_SIZE_BYTES,
        "buffer length does not have expected size"
    );

    unsafe {
        while read_volatile(SDCARD_BUSY as *const i32) != 0 {}
        write_volatile(SDCARD_READ_SECTOR as *mut u32, sector);
        while read_volatile(SDCARD_BUSY as *const i32) != 0 {}
        for byte in buffer_512_bytes.iter_mut() {
            *byte = read_volatile(SDCARD_NEXT_BYTE as *const u8);
        }
    }
}

pub fn sdcard_write_blocking(sector: u32, buffer_512_bytes: &[u8]) {
    assert!(
        buffer_512_bytes.len() == SDCARD_SECTOR_SIZE_BYTES,
        "buffer length does not have expected size"
    );

    unsafe {
        while read_volatile(SDCARD_BUSY as *const i32) != 0 {}
        for byte in buffer_512_bytes {
            write_volatile(SDCARD_NEXT_BYTE as *mut u8, *byte);
        }
        write_volatile(SDCARD_WRITE_SECTOR as *mut u32, sector);
        while read_volatile(SDCARD_BUSY as *const i32) != 0 {}
    }
}
