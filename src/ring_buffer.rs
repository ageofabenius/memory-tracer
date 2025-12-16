use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

const BUFFER_CAPACITY: usize = 4096;

#[derive(Clone, Copy, Debug)]
pub enum AllocatorEvent {
    Allocate { size: usize, ptr_address: usize },
    Free { size: usize, ptr_address: usize },
}

pub struct RingBuffer {
    buffer: [UnsafeCell<AllocatorEvent>; BUFFER_CAPACITY],
    write_index: AtomicUsize,
    read_index: AtomicUsize,
}

unsafe impl Sync for RingBuffer {}

impl RingBuffer {
    pub const fn new() -> Self {
        const ZERO: AllocatorEvent = AllocatorEvent::Free {
            size: 0,
            ptr_address: 0,
        };

        Self {
            buffer: [const { UnsafeCell::new(ZERO) }; BUFFER_CAPACITY],
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, event: AllocatorEvent) {
        let write = self.write_index.load(Ordering::Relaxed);
        let next = (write + 1) % BUFFER_CAPACITY;

        let read = self.read_index.load(Ordering::Acquire);
        if next == read {
            // Buffer is full, drop the event
            return;
        }

        unsafe {
            *self.buffer[write].get() = event;
        }

        self.write_index.store(next, Ordering::Release);
    }

    pub fn pop(&self) -> Option<AllocatorEvent> {
        let read = self.read_index.load(Ordering::Relaxed);
        let write = self.write_index.load(Ordering::Acquire);

        if read == write {
            return None;
        }

        let event = unsafe { *self.buffer[read].get() };

        let next = (read + 1) % BUFFER_CAPACITY;

        self.read_index.store(next, Ordering::Release);

        Some(event)
    }
}
