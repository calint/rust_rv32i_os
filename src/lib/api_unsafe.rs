//
// reviewed: 2025-04-21
//           2026-08-21
//
use super::constants::{
    LED, SDCARD_BUSY, SDCARD_NEXT_BYTE, SDCARD_READ_SECTOR, SDCARD_STATUS, SDCARD_WRITE_SECTOR,
    UART_IN_ADDR, UART_OUT_ADDR,
};
use core::arch::asm;
use core::hint::spin_loop;

pub const SDCARD_SECTOR_SIZE_BYTES: usize = 512;

unsafe extern "C" {
    pub static __heap_start__: u8;
    // note: declared in `linker.ld`
}

pub fn uart_send_byte(byte: u8) {
    unsafe {
        while (UART_OUT_ADDR as *const i32).read_volatile() != -1 {
            spin_loop();
        }
        (UART_OUT_ADDR as *mut u8).write_volatile(byte);
    }
}

#[expect(clippy::cast_possible_truncation, reason = "intended behavior")]
#[expect(clippy::cast_sign_loss, reason = "intended behavior")]
pub fn uart_read_byte() -> u8 {
    unsafe {
        loop {
            let input = (UART_IN_ADDR as *const i32).read_volatile();
            if input != -1 {
                return input as u8;
            }
            spin_loop();
        }
    }
}

pub fn led_set(bits_low_being_on: u32) {
    unsafe { (LED as *mut u32).write_volatile(bits_low_being_on) }
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
    unsafe { (SDCARD_STATUS as *const i32).read_volatile() }
}

pub fn sdcard_read_blocking(sector: u32, buffer_512_bytes: &mut [u8]) {
    assert!(
        buffer_512_bytes.len() == SDCARD_SECTOR_SIZE_BYTES,
        "buffer length does not have expected size"
    );

    unsafe {
        while (SDCARD_BUSY as *const i32).read_volatile() != 0 {
            spin_loop();
        }
        (SDCARD_READ_SECTOR as *mut u32).write_volatile(sector);
        while (SDCARD_BUSY as *const i32).read_volatile() != 0 {
            spin_loop();
        }
        for byte in buffer_512_bytes.iter_mut() {
            *byte = (SDCARD_NEXT_BYTE as *const u8).read_volatile();
        }
    }
}

pub fn sdcard_write_blocking(sector: u32, buffer_512_bytes: &[u8]) {
    assert!(
        buffer_512_bytes.len() == SDCARD_SECTOR_SIZE_BYTES,
        "buffer length does not have expected size"
    );

    unsafe {
        while (SDCARD_BUSY as *const i32).read_volatile() != 0 {
            spin_loop();
        }
        for byte in buffer_512_bytes {
            (SDCARD_NEXT_BYTE as *mut u8).write_volatile(*byte);
        }
        (SDCARD_WRITE_SECTOR as *mut u32).write_volatile(sector);
        while (SDCARD_BUSY as *const i32).read_volatile() != 0 {
            spin_loop();
        }
    }
}
