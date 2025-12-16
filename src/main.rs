use std::{thread::sleep, time::Duration};

use memory_tracer::{
    ring_buffer::RingBuffer, tracing_allocator::TracingAllocator, tracing_printer::TracingPrinter,
};

static RING: RingBuffer = RingBuffer::new();

#[global_allocator]
static GLOBAL: TracingAllocator = TracingAllocator::new(&RING);

fn main() {
    let tracing_printer = TracingPrinter::new(&RING);
    tracing_printer.start();
    println!("Hello world!");
    {
        let _v = vec![1, 2, 3, 4];
    }

    sleep(Duration::from_secs(1));
}
