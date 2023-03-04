#![feature(alloc_error_handler)]
#![feature(core_panic)]

#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
extern crate core;

#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
use std::alloc::Layout;

// `std` has an `#[alloc_error_handler]` with a runtime hook mechanism. `#![no_std]` crates can
// either supply their own implementation, and if they don't a shim will point to `__rdl_oom()`.
//
// Below is the definition of `__rdl_oom()` for demo simplicity reasons. A real implementation
// would have to inject this crate for `#![no_std]` crates and another crate containing `std`'s
// implementation otherwise.
#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
#[alloc_error_handler]
pub unsafe fn on_oom(layout: Layout) -> ! {
    extern "Rust" {
        // This symbol is emitted by rustc next to __rust_alloc_error_handler.
        // Its value depends on the -Zoom={panic,abort} compiler option.
        static __rust_alloc_error_handler_should_panic: u8;
    }

    #[allow(unused_unsafe)]
    if unsafe { __rust_alloc_error_handler_should_panic != 0 } {
        panic!("memory allocation of {} bytes failed", layout.size())
    } else {
        core::panicking::panic_nounwind_fmt(format_args!(
            "memory allocation of {} bytes failed",
            layout.size()
        ))
    }
}
