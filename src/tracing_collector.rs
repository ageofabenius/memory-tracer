use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard},
    thread::{self, sleep},
    time::Duration,
};

use crate::ring_buffer::{AllocatorEvent, RingBuffer};

type AllocationId = usize;

#[derive(Debug, Clone)]
struct AllocatedInterval {
    id: AllocationId,
    start: usize,
    size: usize,
    end_exclusive: usize,
}

impl AllocatedInterval {
    fn new(id: AllocationId, start: usize, size: usize) -> Self {
        Self {
            id,
            start,
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
    ring: &'static RingBuffer,
    allocation_store: AllocationStore,
    index_by_ptr: BTreeMap<usize, AllocationId>,
    changed_since_print: bool,
}

pub struct TracingCollector {
    inner: Arc<Mutex<TracingCollectorInner>>,
}

impl TracingCollector {
    pub fn new(ring: &'static RingBuffer) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TracingCollectorInner {
                ring,
                allocation_store: AllocationStore::new(),
                index_by_ptr: BTreeMap::new(),
                changed_since_print: false,
            })),
        }
    }

    pub fn start(&mut self) {
        let tracing_collector_inner = self.inner.clone();
        thread::spawn(move || {
            loop {
                Self::poll_ring_buffer(tracing_collector_inner.clone());

                sleep(Duration::from_millis(10));
            }
        });
    }

    fn poll_ring_buffer(tracing_collector_inner: Arc<Mutex<TracingCollectorInner>>) {
        let mut inner_lock = tracing_collector_inner.lock().unwrap();
        while let Some(event) = inner_lock.ring.pop() {
            match event {
                AllocatorEvent::Allocate { size, ptr_address } => {
                    Self::record_allocation(&mut inner_lock, size, ptr_address)
                }
                AllocatorEvent::Free { size, ptr_address } => {
                    Self::record_free(&mut inner_lock, size, ptr_address)
                }
            }
            inner_lock.changed_since_print = true;
        }
    }

    fn record_allocation(
        inner: &mut MutexGuard<TracingCollectorInner>,
        size: usize,
        ptr_address: usize,
    ) {
        // println!("allocated {} at 0x{}", size, ptr_address);
        // Determine AllocationId from the store itself
        let allocation_id: AllocationId = inner.allocation_store.push(size, ptr_address);

        // Add to index
        let res = inner.index_by_ptr.insert(ptr_address, allocation_id);
        assert!(res.is_none());
    }

    fn record_free(
        inner: &mut MutexGuard<TracingCollectorInner>,
        _size: usize,
        ptr_address: usize,
    ) {
        // println!("freed {} at 0x{}", _size, ptr_address);
        // Remove from index
        let _allocation_id = inner.index_by_ptr.remove(&ptr_address).unwrap();
    }

    fn print_contents_inner(inner: &mut MutexGuard<TracingCollectorInner>) {
        dbg!(&inner.allocation_store.0);
        dbg!(&inner.index_by_ptr);
        inner.changed_since_print = false
    }

    pub fn print_contents(&self) {
        let mut inner_lock = self.inner.lock().unwrap();
        Self::print_contents_inner(&mut inner_lock);
    }
}
