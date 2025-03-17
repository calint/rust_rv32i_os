#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

mod constants; // FPGA addresses
use constants::*;

// API
#[inline(always)]
fn uart_read_char() -> u8 {
    loop {
        unsafe {
            let input = read_volatile(UART_IN_ADDR as *const i32);
            if input == -1 {
                continue;
            }
            return input as u8;
        }
    }
}

#[inline(always)]
fn uart_send_char(ch: u8) {
    unsafe {
        while read_volatile(UART_OUT_ADDR as *const i32) != -1 {}
        write_volatile(UART_OUT_ADDR as *mut i32, ch as i32);
    }
}

// #[inline(always)]
// fn uart_send_cstr(cstr: *const u8) {
//     unsafe {
//         let mut ptr = cstr;
//         while *ptr != 0 {
//             while read_volatile(UART_OUT_ADDR as *const i32) != -1 {}
//             write_volatile(UART_OUT_ADDR as *mut i32, *ptr as i32);
//             ptr = ptr.offset(1);
//         }
//     }
// }

#[inline(always)]
fn uart_send_str(str: &[u8]) {
    for &byte in str {
        uart_send_char(byte);
    }
}

struct FixedSizeList<T, const N: usize> {
    data: [Option<T>; N],
    count: usize,
}

impl<T: PartialEq + Copy, const N: usize> FixedSizeList<T, N> {
    fn new() -> Self {
        FixedSizeList {
            data: [None; N],
            count: 0,
        }
    }

    fn add(&mut self, item: T) -> bool {
        if self.count < N {
            self.data[self.count] = Some(item);
            self.count += 1;
            true
        } else {
            false
        }
    }

    fn remove(&mut self, item: T) -> bool {
        if let Some(index) = self.data[..self.count]
            .iter()
            .position(|x| x == &Some(item))
        {
            for i in index..self.count - 1 {
                self.data[i] = self.data[i + 1];
            }
            self.data[self.count - 1] = None;
            self.count -= 1;
            true
        } else {
            false
        }
    }

    fn get(&self, index: usize) -> Option<T> {
        self.data[index]
    }

    fn iter(&self) -> FixedSizeListIterator<'_, T, N> {
        FixedSizeListIterator {
            list: self,
            index: 0,
        }
    }
}

struct FixedSizeListIterator<'a, T, const N: usize> {
    list: &'a FixedSizeList<T, N>,
    index: usize,
}

impl<'a, T: Copy, const N: usize> Iterator for FixedSizeListIterator<'a, T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.list.count {
            let item = self.list.data[self.index];
            self.index += 1;
            item
        } else {
            None
        }
    }
}

// setup stack and jump to 'run()'
global_asm!(include_str!("startup.s"));

#[no_mangle]
pub extern "C" fn run() -> ! {
    let mut list: FixedSizeList<&[u8], 5> = FixedSizeList::new();

    list.add(b"hello world");
    list.add(b"echo below:");

    for item in list.iter() {
        uart_send_str(item);
        uart_send_str(b"\r\n");
    }

    loop {
        uart_send_char(uart_read_char());
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
