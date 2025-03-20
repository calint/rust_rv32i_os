use super::api_unsafe::*;
use super::constants::*;

#[inline(always)]
pub fn memory_end() -> u32 {
    MEMORY_END
}

#[inline(always)]
pub fn uart_send_str(str: &[u8]) {
    for &byte in str {
        uart_send_char(byte);
    }
}

pub fn uart_send_hex_u32(i: u32, separate_half_words: bool) {
    uart_send_hex_byte((i >> 24) as u8);
    uart_send_hex_byte((i >> 16) as u8);
    if separate_half_words {
        uart_send_char(b':');
    }
    uart_send_hex_byte((i >> 8) as u8);
    uart_send_hex_byte(i as u8);
}

pub fn uart_send_hex_byte(ch: u8) {
    uart_send_hex_nibble(ch >> 4);
    uart_send_hex_nibble(ch & 0x0f);
}

pub fn uart_send_hex_nibble(nibble: u8) {
    if nibble < 10 {
        uart_send_char(b'0' + nibble);
    } else {
        uart_send_char(b'A' + (nibble - 10));
    }
}
