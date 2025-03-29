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
        // super::api::uart_send_str(b"alloc: at ");
        // super::api::uart_send_hex_u32(aligned_next as u32, true);
        // super::api::uart_send_str(b" size: ");
        // super::api::uart_send_hex_u32(size as u32, true);
        // super::api::uart_send_str(b"\r\n");
        let self_mut_ptr = self as *const Self as *mut Self;
        unsafe {
            (*self_mut_ptr).next = aligned_next + size;
        }
        aligned_next as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // super::api::uart_send_str(b"de-alloc: at ");
        // super::api::uart_send_hex_u32(ptr as u32, true);
        // super::api::uart_send_str(b" size: ");
        // super::api::uart_send_hex_u32(layout.size() as u32, true);
        // super::api::uart_send_str(b"\r\n");
    }
}

#[global_allocator]
static mut ALLOCATOR: BumpAllocator = BumpAllocator { next: 0 };

pub fn allocator_init() {
    unsafe {
        ALLOCATOR.next = &__heap_start__ as *const u8 as usize;
    }
}

pub fn allocator_current_next() -> usize {
    unsafe { ALLOCATOR.next }
}
