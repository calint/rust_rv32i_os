// use super::api::*;
use super::api_unsafe::__heap_start__;
use core::alloc::{GlobalAlloc, Layout};

struct BumpAllocator {
    next: usize,
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let aligned_next = (self.next + align - 1) & !(align - 1);
        // uart_send_str(b"alloc: at ");
        // uart_send_hex_u32(aligned_next as u32, true);
        // uart_send_str(b" size: ");
        // uart_send_hex_u32(size as u32, true);
        // uart_send_str(b"\r\n");
        unsafe {
            (self as *const Self as *mut Self).as_mut().unwrap().next = aligned_next + size;
        }
        aligned_next as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static mut ALLOCATOR: BumpAllocator = BumpAllocator { next: 0 };

pub fn init_bump_allocator() {
    unsafe {
        ALLOCATOR.next = &__heap_start__ as *const u8 as usize;
    }
}
