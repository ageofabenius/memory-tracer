use crate::ring_buffer::{AllocatorEvent, RingBuffer};

use std::{alloc::GlobalAlloc, cell::Cell};

thread_local! {
    static TRACING_ENABLED: Cell<bool> = Cell::new(true);
}

thread_local! {
    static ALLOCATION_CONTEXT: Cell<&'static str> = Cell::new("static");
}

pub struct TracingContext {
    previous: &'static str,
}

impl TracingContext {
    pub fn enter(name: &'static str) -> Self {
        ALLOCATION_CONTEXT.with(|context| {
            let previous = context.get();

            context.set(name);

            Self { previous }
        })
    }

    pub fn exit(&self) {
        ALLOCATION_CONTEXT.with(|context| {
            context.set(self.previous);
        })
    }
}

impl Drop for TracingContext {
    fn drop(&mut self) {
        self.exit();
    }
}

pub struct TracingAllocator {
    ring: &'static RingBuffer,
}

impl TracingAllocator {
    pub const fn new(ring: &'static RingBuffer) -> Self {
        Self { ring }
    }

    pub const fn ring(&self) -> &'static RingBuffer {
        self.ring
    }

    pub fn enable_for_this_thread(&self) {
        TRACING_ENABLED.with(|enabled| enabled.set(true));
    }

    pub fn disable_for_this_thread(&self) {
        TRACING_ENABLED.with(|enabled| enabled.set(false));
    }
}

unsafe impl GlobalAlloc for TracingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let size = layout.size();
        let ptr: *mut u8 = unsafe { libc::malloc(size) } as *mut u8;

        TRACING_ENABLED.with(|enabled| {
            if enabled.get() {
                ALLOCATION_CONTEXT.with(|context| {
                    self.ring.push(AllocatorEvent::Allocate {
                        size,
                        ptr_address: ptr as usize,
                        context: context.get(),
                    })
                });
            }
        });

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        unsafe {
            libc::free(ptr as *mut libc::c_void);
        }

        TRACING_ENABLED.with(|enabled| {
            if enabled.get() {
                self.ring.push(AllocatorEvent::Free {
                    size: layout.size(),
                    ptr_address: ptr as usize,
                });
            }
        });
    }
}
