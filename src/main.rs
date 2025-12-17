use std::{thread::sleep, time::Duration};

use memory_tracer::{
    ring_buffer::RingBuffer,
    tracing_allocator::{TracingAllocator, TracingContext},
    tracing_collector::TracingCollector,
};

static RING: RingBuffer = RingBuffer::new();

#[global_allocator]
static TRACING_ALLOCATOR: TracingAllocator = TracingAllocator::new(&RING);

fn main() {
    let context = TracingContext::enter("main");
    let tracing_collector = TracingCollector::new(&TRACING_ALLOCATOR);
    tracing_collector.start();
    tracing_collector.pretty_print();
    // let v_context = TracingContext::enter("v");
    let _v = vec![1, 2, 3, 4, 5];
    // v_context.exit();
    tracing_collector.pretty_print();
    sleep(Duration::from_secs(1));
    tracing_collector.pretty_print();
    println!("Hello world!");
    {
        let _context = TracingContext::enter("test_vec initial");
        sleep(Duration::from_millis(100));
        tracing_collector.pretty_print();
        let mut test_vec: Vec<usize> = Vec::with_capacity(2);
        test_vec.push(1);
        test_vec.push(2);
        let _context_2 = TracingContext::enter("test_vec reallocate");
        sleep(Duration::from_millis(100));
        tracing_collector.pretty_print();
        println!("{test_vec:?}");
        sleep(Duration::from_millis(100));
        tracing_collector.pretty_print();
        test_vec.push(3);
        println!("{test_vec:?}");
        sleep(Duration::from_millis(100));
        tracing_collector.pretty_print();
    }
    tracing_collector.pretty_print();

    sleep(Duration::from_secs(1));
    tracing_collector.pretty_print();
    context.exit()
}
