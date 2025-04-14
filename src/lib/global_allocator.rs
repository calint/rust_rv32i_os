use super::api::{uart_send_bytes, uart_send_hex_u32};
use super::api_unsafe::{__heap_start__, uart_send_byte};
use core::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr;

// minimum block size and alignment
const MIN_BLOCK_SIZE: usize = 16;

// block metadata structure
struct BlockHeader {
    size: usize,            // total size of the block including header
    is_free: bool,          // whether the block is available for allocation
    next: *mut BlockHeader, // next block in the free list
    prev: *mut BlockHeader, // previous block in the free list
}

pub struct GlobalAllocator {
    free_list: *mut BlockHeader, // head of the free list
}

#[expect(
    clippy::cast_ptr_alignment,
    reason = "Because it is intended behavior."
)]
unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // adjust size to include header and ensure alignment
        let aligned_size = {
            let size = layout.size() + mem::size_of::<BlockHeader>();
            (size + layout.align() - 1) & !(layout.align() - 1)
        };

        // find first suitable free block
        let mut current = self.free_list;

        unsafe {
            while !current.is_null() {
                if (*current).is_free && (*current).size >= aligned_size {
                    // found a suitable block
                    if (*current).size > aligned_size + MIN_BLOCK_SIZE {
                        // split the block if it's significantly larger
                        let remaining_size = (*current).size - aligned_size;
                        let new_block =
                            current.cast::<u8>().add(aligned_size).cast::<BlockHeader>();

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

                    return current.cast::<u8>().add(mem::size_of::<BlockHeader>());
                }

                current = (*current).next;
            }
        }
        // no suitable block found
        ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // uart_send_bytes(b"free: ");
        // uart_send_hex_u32(ptr as u32, true);
        // uart_send_bytes(b" size: ");
        // uart_send_hex_u32(_layout.size() as u32, true);
        // uart_send_bytes(b"\r\n");

        unsafe {
            // get the block header
            let block = ptr.sub(mem::size_of::<BlockHeader>()).cast::<BlockHeader>();

            // mark block as free
            (*block).is_free = true;

            // attempt to merge with adjacent free blocks
            let current = block;

            // merge with next block if possible
            if !(*current).next.is_null()
                && (*(*current).next).is_free
                && ptr::eq(
                    current.cast::<u8>().add((*current).size),
                    (*current).next.cast::<u8>(),
                )
            {
                (*current).size += (*(*current).next).size;
                (*current).next = (*(*current).next).next;
                if !(*current).next.is_null() {
                    (*(*current).next).prev = current;
                }
            }

            // merge with previous block if possible
            if !(*current).prev.is_null()
                && (*(*current).prev).is_free
                && ptr::eq(
                    current.cast::<u8>(),
                    (*current).prev.cast::<u8>().add((*(*current).prev).size),
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

#[expect(
    clippy::cast_ptr_alignment,
    reason = "Because it is intended behavior."
)]
impl GlobalAllocator {
    fn new(memory: *mut u8, total_size: usize) -> Self {
        // initialize the entire memory as one free block
        let first_block = memory.cast::<BlockHeader>();
        unsafe {
            (*first_block).size = total_size;
            (*first_block).is_free = true;
            (*first_block).next = ptr::null_mut();
            (*first_block).prev = ptr::null_mut();
        };

        Self {
            free_list: first_block,
        }
    }

    pub fn init(heap_size: usize) {
        unsafe {
            HEAP_ALLOCATOR = Self::new((&raw const __heap_start__).cast_mut(), heap_size);
        }
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "Because it is intended behavior."
    )]
    pub fn debug_block_list() {
        unsafe {
            let mut current = HEAP_ALLOCATOR.free_list;
            let mut total: usize = 0;
            let mut total_including_headers: usize = 0;
            while !current.is_null() {
                uart_send_bytes(b"at: ");
                uart_send_hex_u32(current as u32, true);
                uart_send_bytes(b", size: ");
                uart_send_hex_u32((*current).size as u32, true);
                if !(*current).is_free {
                    total += (*current).size;
                    total_including_headers += (*current).size + mem::size_of::<BlockHeader>();
                }
                uart_send_bytes(b", free: ");
                uart_send_byte(if (*current).is_free { b'y' } else { b'n' });
                uart_send_bytes(b"\r\n");

                current = (*current).next;
            }
            uart_send_bytes(b"total user allocated: ");
            uart_send_hex_u32(total as u32, true);
            uart_send_bytes(b" bytes\r\n");
            uart_send_bytes(b"total allocated including headers: ");
            uart_send_hex_u32(total_including_headers as u32, true);
            uart_send_bytes(b" bytes\r\n");
        }
    }
}

#[global_allocator]
static mut HEAP_ALLOCATOR: GlobalAllocator = GlobalAllocator {
    free_list: ptr::null_mut(),
};
