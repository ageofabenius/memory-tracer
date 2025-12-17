use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard},
    thread::{self, sleep},
    time::Duration,
};

use crate::{ring_buffer::AllocatorEvent, tracing_allocator::TracingAllocator};

type AllocationId = usize;

#[derive(Debug, Clone)]
pub struct AllocatedInterval {
    id: AllocationId,
    start_address: usize,
    size: usize,
    end_exclusive: usize,
}

// #[derive(Debug, Clone)]
// pub struct GapInterval {
//     start_address: usize,
//     size: usize,
//     end_exclusive: usize,
// }

// #[derive(Debug, Clone)]
// pub enum MemoryInterval {
//     Allocated(AllocatedInterval),
//     Gap(GapInterval),
// }

impl AllocatedInterval {
    fn new(id: AllocationId, start: usize, size: usize) -> Self {
        Self {
            id,
            start_address: start,
            size,
            end_exclusive: start + size,
        }
    }
}

/// An append-only store of all recorded allocations.
struct AllocationStore(Vec<AllocatedInterval>);

impl AllocationStore {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    fn get(&self, id: AllocationId) -> Option<&AllocatedInterval> {
        self.0.get(id)
    }

    fn push(&mut self, size: usize, ptr_address: usize) -> AllocationId {
        let this_id = self.0.len();
        self.0
            .push(AllocatedInterval::new(this_id, ptr_address, size));
        this_id
    }
}

pub struct TracingCollectorInner {
    tracing_allocator: &'static TracingAllocator,
    allocation_store: AllocationStore,
    index_by_ptr: BTreeMap<usize, AllocationId>,
    stop_flag: bool,
}

pub struct TracingCollector {
    inner: Arc<Mutex<TracingCollectorInner>>,
}

impl TracingCollector {
    pub fn new(tracing_allocator: &'static TracingAllocator) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TracingCollectorInner {
                tracing_allocator,
                allocation_store: AllocationStore::new(),
                index_by_ptr: BTreeMap::new(),
                stop_flag: false,
            })),
        }
    }

    pub fn start(&self) {
        {
            // println!("Setting inner_lock.stop_flag = false");
            let mut inner_lock = self.inner.lock().unwrap();
            inner_lock.stop_flag = false;
            // println!("Set inner_lock.stop_flag = false");
        }

        // println!("Dropped initial lock");
        let tracing_collector_inner = self.inner.clone();
        thread::spawn(move || {
            {
                // IMPORTANT: Disable tracing for this thread.  If enabled,
                // TracingAllocator will publish events for every println
                // statement (these allocate) within this thread.
                // println!("Disabling tracing for TracingCollector polling thread");
                tracing_collector_inner
                    .lock()
                    .unwrap()
                    .tracing_allocator
                    .disable_for_this_thread();
                // println!("Disabled tracing for TracingCollector polling thread");
            }

            // println!("Started polling thread");
            loop {
                // Perform all mutex interactions in a new scope so the lock is
                // dropped before the call to sleep
                {
                    // println!("Top of polling loop");
                    let mut inner_lock = tracing_collector_inner.lock().unwrap();
                    // println!("Acquired polling lock");

                    if inner_lock.stop_flag {
                        return;
                    }

                    // println!("Polling ring buffer...");
                    Self::poll_ring_buffer(&mut inner_lock);
                }

                sleep(Duration::from_millis(100));
            }
        });
    }

    pub fn stop(&self) {
        // println!("Acquiring lock to stop TracingCollector");
        let mut inner_lock = self.inner.lock().unwrap();
        inner_lock.stop_flag = true;
        // println!("Stopped TracingCollector");
    }

    fn poll_ring_buffer(inner_lock: &mut MutexGuard<TracingCollectorInner>) {
        // println!("Polling ring buffer...");
        // let mut n = 0;
        while let Some(event) = inner_lock.tracing_allocator.ring().pop() {
            // println!("Popped an event, number: {n}");
            match event {
                AllocatorEvent::Allocate { size, ptr_address } => {
                    Self::record_allocation(inner_lock, size, ptr_address)
                }
                AllocatorEvent::Free { size, ptr_address } => {
                    Self::record_free(inner_lock, size, ptr_address)
                }
            }
            // n += 1;
        }
        // if n > 0 {
        //     println!("Collected {n} events this polling loop");
        // }
    }

    fn record_allocation(
        inner_lock: &mut MutexGuard<TracingCollectorInner>,
        size: usize,
        ptr_address: usize,
    ) {
        // println!("Recording allocation");
        // Determine AllocationId from the store itself
        let allocation_id: AllocationId = inner_lock.allocation_store.push(size, ptr_address);

        // Add to index
        let res = inner_lock.index_by_ptr.insert(ptr_address, allocation_id);
        assert!(res.is_none());
    }

    fn record_free(
        inner_lock: &mut MutexGuard<TracingCollectorInner>,
        _size: usize,
        ptr_address: usize,
    ) {
        // println!("Recording freeing");
        // Remove from index
        let _allocation_id = inner_lock.index_by_ptr.remove(&ptr_address).unwrap();
    }

    pub fn print_contents(&self) {
        // println!("Acquiring lock to print");
        let mut inner_lock = self.inner.lock().unwrap();
        // println!("Acquired lock to print");
        Self::print_contents_inner(&mut inner_lock);
    }

    fn print_contents_inner(inner_lock: &mut MutexGuard<TracingCollectorInner>) {
        dbg!(&inner_lock.allocation_store.0);
        dbg!(&inner_lock.index_by_ptr);
    }

    pub fn get_allocated_intervals(&self) -> Vec<AllocatedInterval> {
        let mut inner_lock = self.inner.lock().unwrap();
        Self::get_allocated_intervals_inner(&mut inner_lock)
    }

    fn get_allocated_intervals_inner(
        inner_lock: &mut MutexGuard<TracingCollectorInner>,
    ) -> Vec<AllocatedInterval> {
        let mut memory_intervals = Vec::new();
        for alloc_id in &mut inner_lock.index_by_ptr.values() {
            let allocation = inner_lock.allocation_store.get(*alloc_id).unwrap();
            memory_intervals.push(allocation.clone());
        }

        memory_intervals
    }
}
