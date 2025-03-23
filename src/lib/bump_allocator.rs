use core::alloc::{GlobalAlloc, Layout};

struct BumpAllocator {
    next: usize,
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let aligned_next = (self.next + align - 1) & !(align - 1);
        unsafe {
            (self as *const Self as *mut Self).as_mut().unwrap().next = aligned_next + size;
        }
        aligned_next as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static mut ALLOCATOR: BumpAllocator = BumpAllocator { next: 0x1_0000 };
