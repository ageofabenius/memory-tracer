use crate::ring_buffer::{AllocatorEvent, RingBuffer};

use std::alloc::GlobalAlloc;

pub struct TracingAllocator {
    ring: &'static RingBuffer,
}

impl TracingAllocator {
    pub const fn new(ring: &'static RingBuffer) -> Self {
        Self { ring }
    }
}

unsafe impl GlobalAlloc for TracingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let size = layout.size();
        let ptr: *mut u8 = unsafe { libc::malloc(size) } as *mut u8;

        self.ring.push(AllocatorEvent::Allocate {
            size,
            ptr_address: ptr as usize,
        });

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        unsafe {
            libc::free(ptr as *mut libc::c_void);
        }

        self.ring.push(AllocatorEvent::Free {
            size: layout.size(),
            ptr_address: ptr as usize,
        });
    }
}
