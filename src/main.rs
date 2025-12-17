use std::{thread::sleep, time::Duration};

use memory_tracer::{
    tracing_collector::TracingCollector, ring_buffer::RingBuffer, tracing_allocator::TracingAllocator,
};

static RING: RingBuffer = RingBuffer::new();

#[global_allocator]
static GLOBAL: TracingAllocator = TracingAllocator::new(&RING);

fn main() {
    let tracing_collector = TracingCollector::new(&RING);
    tracing_collector.start();
    tracing_collector.print_contents();
    let _v = vec![1, 2, 3, 4, 5];
    tracing_collector.print_contents();
    sleep(Duration::from_secs(1));
    tracing_collector.print_contents();
    println!("Hello world!");
    {
        let _v = vec![1, 2, 3, 4];
    }
    tracing_collector.print_contents();

    sleep(Duration::from_secs(1));
    tracing_collector.print_contents();
}
