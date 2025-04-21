//
// reviewed: 2025-04-21
//
use super::api_unsafe::{__heap_start__, uart_send_byte};
use super::constants::MEMORY_END;

pub struct Memory;

impl Memory {
    pub const fn end() -> u32 {
        MEMORY_END
    }

    pub fn heap_start() -> u32 {
        &raw const __heap_start__ as u32
    }
}

pub fn u8_slice_to_u32(number_as_str: &[u8]) -> u32 {
    let mut num = 0;
    for &ch in number_as_str {
        if !ch.is_ascii_digit() {
            break;
        }
        num = num * 10 + u32::from(ch - b'0');
    }
    num
}

pub fn u8_slice_bits_to_u32(binary_as_str: &[u8]) -> u32 {
    if binary_as_str.is_empty() {
        return 0;
    }
    let mut num = 0;
    let mut bit_value = 1 << (binary_as_str.len() - 1);
    for &ch in binary_as_str {
        if ch != b'0' && ch != b'1' {
            break;
        }
        if ch == b'1' {
            num += bit_value;
        }
        bit_value >>= 1;
    }
    num
}

pub trait Printer {
    /// Prints a byte.
    fn pb(&self, byte: u8);

    /// Prints implementation specific new line.
    fn nl(&self);

    /// Prints a slice of bytes.
    fn p(&self, bytes: &[u8]) {
        for &byte in bytes {
            self.pb(byte);
        }
    }

    /// Prints implementation specific multiple new lines.
    fn nlc(&self, count: usize) {
        for _ in 0..count {
            self.nl();
        }
    }

    /// Prints a slice of bytes followed by implementation specific new line.
    fn pl(&self, bytes: &[u8]) {
        self.p(bytes);
        self.nl();
    }

    /// Prints a 4-bit unsigned integer as hexadecimal.
    fn p_hex_nibble(&self, nibble: u8) {
        if nibble < 10 {
            self.pb(b'0' + nibble);
        } else {
            self.pb(b'A' + (nibble - 10));
        }
    }

    /// Prints a 8-bit unsigned integer as hexadecimal.
    fn p_hex_u8(&self, i: u8) {
        self.p_hex_nibble(i >> 4);
        self.p_hex_nibble(i & 0x0f);
    }

    /// Prints a 32-bit unsigned integer as hexadecimal.
    #[allow(clippy::cast_possible_truncation, reason = "intended behavior")]
    fn p_hex_u32(&self, i: u32, separate_half_words: bool) {
        self.p_hex_u8((i >> 24) as u8);
        self.p_hex_u8((i >> 16) as u8);
        if separate_half_words {
            self.pb(b':');
        }
        self.p_hex_u8((i >> 8) as u8);
        self.p_hex_u8(i as u8);
    }

    /// Prints a 32-bit unsigned integer.
    fn p_u32(&self, num: u32) {
        let mut n = num;
        let mut digits = [0_u8; 10];
        let mut i = 0;
        while n > 0 {
            digits[i] = b'0' + (n % 10) as u8;
            n /= 10;
            i += 1;
        }
        if i == 0 {
            self.pb(b'0');
            return;
        }
        for &b in digits.iter().rev() {
            self.pb(b);
        }
    }
}

pub struct PrinterUART;

impl PrinterUART {
    pub const fn new() -> Self {
        Self
    }
}

impl Printer for PrinterUART {
    fn pb(&self, byte: u8) {
        uart_send_byte(byte);
    }

    fn nl(&self) {
        self.p(b"\r\n");
    }
}

/// A printer that ignores all output.
pub struct PrinterVoid;

impl PrinterVoid {
    pub const fn new() -> Self {
        Self
    }
}

impl Printer for PrinterVoid {
    fn pb(&self, _: u8) {}
    fn nl(&self) {}
    fn nlc(&self, _: usize) {}
    fn p(&self, _: &[u8]) {}
    fn pl(&self, _: &[u8]) {}
    fn p_hex_nibble(&self, _: u8) {}
    fn p_hex_u8(&self, _: u8) {}
    fn p_hex_u32(&self, _: u32, _: bool) {}
    fn p_u32(&self, _: u32) {}
}
