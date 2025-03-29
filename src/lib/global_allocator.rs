use super::api::{uart_send_bytes, uart_send_hex_u32};
use super::api_unsafe::{__heap_start__, uart_send_byte};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

// Minimum block size and alignment
const MIN_BLOCK_SIZE: usize = 16;
const ALIGNMENT: usize = 8;

// Block metadata structure
struct BlockHeader {
    size: usize,            // Total size of the block including header
    is_free: bool,          // Whether the block is available for allocation
    next: *mut BlockHeader, // Next block in the free list
    prev: *mut BlockHeader, // Previous block in the free list
}

pub struct GlobalAllocator {
    free_list: *mut BlockHeader, // Head of the free list
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Adjust size to include header and ensure alignment
        let aligned_size = {
            let size = layout.size() + core::mem::size_of::<BlockHeader>();
            (size + ALIGNMENT - 1) & !(ALIGNMENT - 1)
        };

        // Find first suitable free block
        let mut current = self.free_list;

        unsafe {
            while !current.is_null() {
                if (*current).is_free && (*current).size >= aligned_size {
                    // Found a suitable block
                    if (*current).size > aligned_size + MIN_BLOCK_SIZE {
                        // Split the block if it's significantly larger
                        let remaining_size = (*current).size - aligned_size;
                        let new_block = (current as *mut u8).add(aligned_size) as *mut BlockHeader;

                        (*new_block).size = remaining_size;
                        (*new_block).is_free = true;
                        (*new_block).next = (*current).next;
                        (*new_block).prev = current;

                        if !(*current).next.is_null() {
                            (*(*current).next).prev = new_block;
                        }

                        (*current).size = aligned_size;
                        (*current).next = new_block;
                    }

                    (*current).is_free = false;

                    // uart_send_bytes(b"alloc: ");
                    // uart_send_hex_u32(current as u32, true);
                    // uart_send_bytes(b" size: ");
                    // uart_send_hex_u32(request_size as u32, true);
                    // uart_send_bytes(b"\r\n");

                    return (current as *mut u8).add(core::mem::size_of::<BlockHeader>());
                }

                current = (*current).next;
            }
        }
        // No suitable block found
        ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // uart_send_bytes(b"free: ");
        // uart_send_hex_u32(ptr as u32, true);
        // uart_send_bytes(b" size: ");
        // uart_send_hex_u32(_layout.size() as u32, true);
        // uart_send_bytes(b"\r\n");

        unsafe {
            // Get the block header
            let block = ptr.sub(core::mem::size_of::<BlockHeader>()) as *mut BlockHeader;

            // Mark block as free
            (*block).is_free = true;

            // Attempt to merge with adjacent free blocks
            let current = block;

            // Merge with next block if possible
            if !(*current).next.is_null()
                && (*(*current).next).is_free
                && core::ptr::eq(
                    (current as *mut u8).add((*current).size),
                    (*current).next as *mut u8,
                )
            {
                (*current).size += (*(*current).next).size;
                (*current).next = (*(*current).next).next;
                if !(*current).next.is_null() {
                    (*(*current).next).prev = current;
                }
            }

            // Merge with previous block if possible
            if !(*current).prev.is_null()
                && (*(*current).prev).is_free
                && core::ptr::eq(
                    current as *mut u8,
                    ((*current).prev as *mut u8).add((*(*current).prev).size),
                )
            {
                (*(*current).prev).size += (*current).size;
                (*(*current).prev).next = (*current).next;
                if !(*current).next.is_null() {
                    (*(*current).next).prev = (*current).prev;
                }
            }
        }
    }
}

impl GlobalAllocator {
    fn new(memory: *mut u8, total_size: usize) -> Self {
        // Initialize the entire memory as one free block
        let first_block = memory as *mut BlockHeader;
        unsafe {
            (*first_block).size = total_size;
            (*first_block).is_free = true;
            (*first_block).next = ptr::null_mut();
            (*first_block).prev = ptr::null_mut();
        }

        GlobalAllocator {
            free_list: first_block,
        }
    }
}

// Implement a global allocator for no_std usage
#[global_allocator]
static mut HEAP_ALLOCATOR: GlobalAllocator = GlobalAllocator {
    free_list: ptr::null_mut(),
};

// Example initialization function
pub fn global_allocator_init(heap_size: usize) {
    unsafe {
        HEAP_ALLOCATOR = GlobalAllocator::new(&__heap_start__ as *const u8 as *mut u8, heap_size);
    }
}

pub fn global_allocator_debug_block_list() {
    unsafe {
        let mut current = HEAP_ALLOCATOR.free_list;
        let mut total: usize = 0;
        while !current.is_null() {
            uart_send_bytes(b"at: ");
            uart_send_hex_u32(current as u32, true);
            uart_send_bytes(b", size: ");
            uart_send_hex_u32((*current).size as u32, true);
            if !(*current).is_free {
                total += (*current).size;
            }
            uart_send_bytes(b", free: ");
            uart_send_byte(if (*current).is_free { b'y' } else { b'n' });
            uart_send_bytes(b"\r\n");

            current = (*current).next;
        }
        uart_send_bytes(b"total: ");
        uart_send_hex_u32(total as u32, true);
        uart_send_bytes(b"\r\n");
    }
}
