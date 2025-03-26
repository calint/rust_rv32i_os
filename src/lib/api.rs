use super::api_unsafe::*;
use super::constants::*;

pub fn memory_end() -> u32 {
    MEMORY_END
}

pub fn memory_heap_start() -> u32 {
    &raw const super::api::__heap_start__ as u32
}

pub fn uart_send_bytes(s: &[u8]) {
    for &byte in s {
        uart_send_byte(byte);
    }
}

pub fn uart_send_cstr(s: &[u8]) {
    for &byte in s {
        if byte == 0 {
            break;
        }
        uart_send_byte(byte);
    }
}

pub fn uart_send_hex_u32(i: u32, separate_half_words: bool) {
    uart_send_hex_byte((i >> 24) as u8);
    uart_send_hex_byte((i >> 16) as u8);
    if separate_half_words {
        uart_send_byte(b':');
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
        uart_send_byte(b'0' + nibble);
    } else {
        uart_send_byte(b'A' + (nibble - 10));
    }
}

pub fn uart_send_move_back(count: usize) {
    for _ in 0..count {
        uart_send_byte(8);
    }
}

pub fn u8_slice_to_u32(number_as_str: &[u8]) -> u32 {
    let mut num = 0;
    for &ch in number_as_str {
        if ch < b'0' || ch > b'9' {
            return num;
        }
        num = num * 10 + (ch - b'0') as u32;
    }
    num
}
