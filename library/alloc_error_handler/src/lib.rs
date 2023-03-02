#![feature(alloc_error_handler)]
#![feature(core_panic)]
// we can't even build an empty crate as no_std dylib until after bootstrapping;
// it'll be missing eh_personality and the compiler doesn't know that's okay
// mattmatt maaaaybe we don't actually need no_std at all
// #![cfg_attr(all(not(bootstrap), feature = "unified-sysroot-injection"), no_std)]

// depending on std, but need to get core::panicking::panic_nounwind_fmt
#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
extern crate core;

#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
use std::alloc::Layout;

// if we need no_std use below
//#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
//use alloc::alloc::Layout;

// mattmatt below is __rdl_oom() which i guess is just for no_std crates?
// library/std/src/alloc.rs has a #[alloc_error_handler] impl which has runtime hooks
//
// probably would need to supply normal crates with the hook behavior, but would need to supply the
// __rdl_oom impl to no_std crates. either keep the bad codegen shim or inject a different extern
// statement depending on whether local crate is no_std

#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
#[cfg_attr(all(not(bootstrap), feature = "unified-sysroot-injection"), alloc_error_handler)]
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
