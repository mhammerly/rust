#![cfg_attr(all(not(bootstrap), feature = "unified-sysroot-injection"), feature(panic_can_unwind))]
#![cfg_attr(all(not(bootstrap), feature = "unified-sysroot-injection"), feature(panic_info_message))]
#![cfg_attr(all(not(bootstrap), feature = "unified-sysroot-injection"), feature(std_internals))]

#![cfg_attr(all(not(bootstrap), feature = "unified-sysroot-injection"), feature(needs_panic_runtime))]
#![cfg_attr(all(not(bootstrap), feature = "unified-sysroot-injection"), needs_panic_runtime)]

// Library features (unwind):
#![cfg_attr(all(not(bootstrap), feature = "unified-sysroot-injection"), feature(panic_unwind))]

// Ensure that std can be linked against panic_abort despite compiled with `-C panic=unwind`
// This is present in std but I get an unknown lint error here
// #![cfg_attr(all(not(bootstrap), feature = "unified-sysroot-injection"), deny(ffi_unwind_calls))]

#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
use core::panic::{BoxMeUp, PanicInfo};
#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
use std::rust_panic_with_hook;
#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
use core::mem;

#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
use std::any::Any;

#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
use std::fmt;

/// Entry point of panics from the core crate (`panic_impl` lang item).
#[cfg(not(test))]
#[cfg(all(not(bootstrap), feature = "unified-sysroot-injection"))]
#[panic_handler]
pub fn begin_panic_handler(info: &PanicInfo<'_>) -> ! {
    struct PanicPayload<'a> {
        inner: &'a fmt::Arguments<'a>,
        string: Option<String>,
    }

    impl<'a> PanicPayload<'a> {
        fn new(inner: &'a fmt::Arguments<'a>) -> PanicPayload<'a> {
            PanicPayload { inner, string: None }
        }

        fn fill(&mut self) -> &mut String {
            use std::fmt::Write;

            let inner = self.inner;
            // Lazily, the first time this gets called, run the actual string formatting.
            self.string.get_or_insert_with(|| {
                let mut s = String::new();
                drop(s.write_fmt(*inner));
                s
            })
        }
    }

    unsafe impl<'a> BoxMeUp for PanicPayload<'a> {
        fn take_box(&mut self) -> *mut (dyn Any + Send) {
            // We do two allocations here, unfortunately. But (a) they're required with the current
            // scheme, and (b) we don't handle panic + OOM properly anyway (see comment in
            // begin_panic below).
            let contents = mem::take(self.fill());
            Box::into_raw(Box::new(contents))
        }

        fn get(&mut self) -> &(dyn Any + Send) {
            self.fill()
        }
    }

    struct StrPanicPayload(&'static str);

    unsafe impl BoxMeUp for StrPanicPayload {
        fn take_box(&mut self) -> *mut (dyn Any + Send) {
            Box::into_raw(Box::new(self.0))
        }

        fn get(&mut self) -> &(dyn Any + Send) {
            &self.0
        }
    }

    let loc = info.location().unwrap(); // The current implementation always returns Some
    let msg = info.message().unwrap(); // The current implementation always returns Some
    std::__rust_end_short_backtrace(move || { // mattmatt
                                              // std::sys_common::backtrace::__rust_end_short_backtrace
        if let Some(msg) = msg.as_str() {
            rust_panic_with_hook(&mut StrPanicPayload(msg), info.message(), loc, info.can_unwind());
        } else {
            rust_panic_with_hook(
                &mut PanicPayload::new(msg),
                info.message(),
                loc,
                info.can_unwind(),
            );
        }
    })
}
