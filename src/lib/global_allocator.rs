//
// reviewed: 2025-04-21
//           2026-08-21
//
use super::api::{Memory, Printer};
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem;
use core::ptr;

// note: align to ensure u64 is aligned on 8 bytes
//       minimum allocation uses 16 + 8 bytes
#[repr(C, align(8))]
struct BlockHeader {
    next: *mut Self, // Pointer to the next block in the list.
    prev: *mut Self, // Pointer to the previous block in the list.
    size: usize,     // Total size of the block, including the header.
    is_free: bool,   // Indicates whether the block is available for allocation.
}

const MIN_BLOCK_SIZE: usize = mem::size_of::<BlockHeader>() * 2;

pub struct GlobalAllocator {
    block_head: UnsafeCell<*mut BlockHeader>,
    ram_size_bytes: UnsafeCell<usize>,
}

// SAFETY: Single-threaded embedded target without concurrent allocator calls.
unsafe impl Sync for GlobalAllocator {}

#[global_allocator]
static HEAP_ALLOCATOR: GlobalAllocator = GlobalAllocator {
    block_head: UnsafeCell::new(ptr::null_mut()),
    ram_size_bytes: UnsafeCell::new(0),
};

#[expect(clippy::cast_ptr_alignment, reason = "intended behavior")]
unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = mem::align_of::<BlockHeader>();
        let header_size = mem::size_of::<BlockHeader>();
        let aligned_size = (layout.size() + header_size).next_multiple_of(align);

        // find first suitable free block
        unsafe {
            let mut current = *self.block_head.get();

            while !current.is_null() {
                if (*current).is_free && (*current).size >= aligned_size {
                    // found a suitable block
                    if (*current).size > aligned_size + MIN_BLOCK_SIZE {
                        // split the block if it's significantly larger
                        let remaining_size = (*current).size - aligned_size;
                        let new_block =
                            current.cast::<u8>().add(aligned_size).cast::<BlockHeader>();

                        *new_block = BlockHeader {
                            next: (*current).next,
                            prev: current,
                            size: remaining_size,
                            is_free: true,
                        };

                        if !(*current).next.is_null() {
                            (*(*current).next).prev = new_block;
                        }

                        (*current).size = aligned_size;
                        (*current).next = new_block;
                    }

                    (*current).is_free = false;

                    return current.cast::<u8>().add(mem::size_of::<BlockHeader>());
                }

                current = (*current).next;
            }
        }

        // no suitable block found
        panic!("out of heap space");
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        unsafe {
            // get the block header
            let block = ptr.sub(mem::size_of::<BlockHeader>()).cast::<BlockHeader>();

            // mark block as free
            (*block).is_free = true;

            // attempt to merge with adjacent free blocks
            let current = block;

            // merge with next block if possible
            if !(*current).next.is_null() && (*(*current).next).is_free {
                (*current).size += (*(*current).next).size;
                (*current).next = (*(*current).next).next;
                if !(*current).next.is_null() {
                    (*(*current).next).prev = current;
                }
            }

            // merge with previous block if possible
            if !(*current).prev.is_null() && (*(*current).prev).is_free {
                (*(*current).prev).size += (*current).size;
                (*(*current).prev).next = (*current).next;
                if !(*current).next.is_null() {
                    (*(*current).next).prev = (*current).prev;
                }
            }
        }
    }
}

#[expect(clippy::cast_ptr_alignment, reason = "intended behavior")]
impl GlobalAllocator {
    /// Called once at start of program.
    pub fn init() {
        // align heap start upward to align_of::<BlockHeader>()
        let raw_start = Memory::heap_start() as usize;
        let raw_end = Memory::end() as usize;
        let align = mem::align_of::<BlockHeader>();
        let aligned_start = raw_start.next_multiple_of(align);
        let usable_size = raw_end - aligned_start;

        let first_block = (aligned_start as *mut u8).cast::<BlockHeader>();
        unsafe {
            *first_block = BlockHeader {
                next: ptr::null_mut(),
                prev: ptr::null_mut(),
                size: usable_size,
                is_free: true,
            };
            *HEAP_ALLOCATOR.block_head.get() = first_block;
            *HEAP_ALLOCATOR.ram_size_bytes.get() = usable_size;
        }
    }

    #[expect(clippy::cast_possible_truncation, reason = "intended behavior")]
    pub fn debug_block_list(printer: &dyn Printer) {
        unsafe {
            let mut current = *HEAP_ALLOCATOR.block_head.get();
            let mut total_user_allocated: usize = 0;
            let mut total_allocated_with_headers: usize = 0;
            while !current.is_null() {
                printer.p_hex_u32(current as u32, true);
                printer.p(b", size: ");
                printer.p_hex_u32((*current).size as u32, true);
                if !(*current).is_free {
                    total_allocated_with_headers += (*current).size;
                    total_user_allocated += (*current).size - mem::size_of::<BlockHeader>();
                }
                printer.p(b", free: ");
                printer.pb(if (*current).is_free { b'y' } else { b'n' });
                printer.nl();

                current = (*current).next;
            }
            printer.nl();
            printer.p(b"ram size: ");
            printer.p_u32(*HEAP_ALLOCATOR.ram_size_bytes.get() as u32);
            printer.pl(b" bytes");
            printer.p(b"total user allocated: ");
            printer.p_u32(total_user_allocated as u32);
            printer.pl(b" bytes");
            printer.p(b"total allocated including headers: ");
            printer.p_u32(total_allocated_with_headers as u32);
            printer.pl(b" bytes");
            printer.p(b"block header size: ");
            printer.p_u32(mem::size_of::<BlockHeader>() as u32);
            printer.pl(b" bytes");
            printer.p(b"min block size: ");
            printer.p_u32(MIN_BLOCK_SIZE as u32);
            printer.pl(b" bytes");
        }
    }
}
