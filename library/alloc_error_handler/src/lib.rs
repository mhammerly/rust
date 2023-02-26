#![feature(alloc_error_handler)]
#![feature(core_panic)]
#![no_std]

use alloc::alloc::Layout;

// mattmatt below is __rdl_oom() which i guess is just for no_std crates?
// library/std/src/alloc.rs has a #[alloc_error_handler] impl which has runtime hooks
//
// probably would need to supply normal crates with the hook behavior, but would need to supply the
// __rdl_oom impl to no_std crates. either keep the bad codegen shim or inject a different extern
// statement depending on whether local crate is no_std

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
