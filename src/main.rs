use std::{thread::sleep, time::Duration};

use memory_tracer::{
    ring_buffer::RingBuffer, tracing_allocator::TracingAllocator,
    tracing_collector::TracingCollector,
};

static RING: RingBuffer = RingBuffer::new();

#[global_allocator]
static TRACING_ALLOCATOR: TracingAllocator = TracingAllocator::new(&RING);

fn main() {
    let tracing_collector = TracingCollector::new(&TRACING_ALLOCATOR);
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
    tracing_collector.stop();
}
