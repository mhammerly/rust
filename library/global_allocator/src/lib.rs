#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
use std::alloc::{GlobalAlloc, Layout, System};

#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
struct DefaultAllocator;

#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
unsafe impl GlobalAlloc for DefaultAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
#[global_allocator]
static GLOBAL: DefaultAllocator = DefaultAllocator;
