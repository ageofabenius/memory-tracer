use std::{
    thread::{self, sleep},
    time::Duration,
};

use crate::ring_buffer::{AllocatorEvent, RingBuffer};

pub struct TracingPrinter {
    ring: &'static RingBuffer,
}

impl TracingPrinter {
    pub const fn new(ring: &'static RingBuffer) -> Self {
        Self { ring }
    }

    pub fn start(self) {
        thread::spawn(move || {
            loop {
                self.drain_ring();

                sleep(Duration::from_millis(100));
            }
        });
    }

    fn drain_ring(&self) {
        while let Some(event) = self.ring.pop() {
            let (event_type, size, ptr) = match event {
                AllocatorEvent::Allocate {
                    size,
                    ptr_address,
                    context: _,
                } => ("allocated", size, ptr_address),
                AllocatorEvent::Free { size, ptr_address } => ("freed", size, ptr_address),
            };
            println!("{} {} bytes at 0x{}", event_type, size, ptr);
        }
    }
}
