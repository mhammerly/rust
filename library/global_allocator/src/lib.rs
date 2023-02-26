use std::alloc::{GlobalAlloc, Layout, System};

struct DefaultAllocator;

unsafe impl GlobalAlloc for DefaultAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: DefaultAllocator = DefaultAllocator;
