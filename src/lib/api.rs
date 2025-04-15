use super::api_unsafe::{__heap_start__, uart_send_byte};
use super::constants::MEMORY_END;

pub const fn memory_end() -> u32 {
    MEMORY_END
}

pub fn memory_heap_start() -> u32 {
    &raw const __heap_start__ as u32
}

pub fn u8_slice_to_u32(number_as_str: &[u8]) -> u32 {
    let mut num = 0;
    for &ch in number_as_str {
        if !ch.is_ascii_digit() {
            return num;
        }
        num = num * 10 + u32::from(ch - b'0');
    }
    num
}

pub trait Printer {
    /// Prints a byte.
    fn pb(&self, byte: u8);

    /// Prints a slice of bytes.
    fn p(&self, bytes: &[u8]);

    /// Prints a slice of bytes followed by implementation specific new line.
    fn pl(&self, bytes: &[u8]);

    /// Prints a 4-bit unsigned integer as hexadecimal.
    fn p_hex_nibble(&self, nibble: u8);

    /// Prints a 8-bit unsigned integer as hexadecimal.
    fn p_hex_u8(&self, i: u8);

    /// Prints a 32-bit unsigned integer as hexadecimal.
    fn p_hex_u32(&self, i: u32, separate_half_words: bool);
}

pub struct PrinterUART;

impl PrinterUART {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Printer for PrinterUART {
    fn pb(&self, byte: u8) {
        uart_send_byte(byte);
    }

    fn p(&self, bytes: &[u8]) {
        for &byte in bytes {
            uart_send_byte(byte);
        }
    }

    fn pl(&self, bytes: &[u8]) {
        self.p(bytes);
        self.p(b"\r\n");
    }

    fn p_hex_nibble(&self, nibble: u8) {
        if nibble < 10 {
            self.pb(b'0' + nibble);
        } else {
            self.pb(b'A' + (nibble - 10));
        }
    }

    fn p_hex_u8(&self, i: u8) {
        self.p_hex_nibble(i >> 4);
        self.p_hex_nibble(i & 0x0f);
    }

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
}

pub struct PrinterVoid;

impl PrinterVoid {
    pub const fn new() -> Self {
        Self {}
    }
}

/// Ignores all `Printer` methods.
impl Printer for PrinterVoid {
    fn pb(&self, _: u8) {}
    fn p(&self, _: &[u8]) {}
    fn pl(&self, _: &[u8]) {}
    fn p_hex_nibble(&self, _: u8) {}
    fn p_hex_u8(&self, _: u8) {}
    fn p_hex_u32(&self, _: u32, _: bool) {}
}
