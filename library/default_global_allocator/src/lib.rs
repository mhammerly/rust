use std::alloc::System;

#[global_allocator]
static DEFAULT_ALLOC: System = System;
