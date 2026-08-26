use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::qjs;

#[derive(Debug, Default)]
struct ExternalMemoryTrackerInner {
    bytes: AtomicUsize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ExternalMemoryTracker {
    inner: Arc<ExternalMemoryTrackerInner>,
}

impl ExternalMemoryTracker {
    pub(crate) fn allocate(&self, bytes: usize) -> ExternalMemoryAllocation {
        self.add(bytes);
        ExternalMemoryAllocation {
            tracker: self.clone(),
            bytes,
        }
    }

    pub(crate) fn bytes(&self) -> usize {
        self.inner.bytes.load(Ordering::Acquire)
    }

    fn add(&self, bytes: usize) {
        update_bytes(&self.inner.bytes, |current| {
            current
                .checked_add(bytes)
                .expect("external memory accounting overflow")
        });
    }

    fn subtract(&self, bytes: usize) {
        update_bytes(&self.inner.bytes, |current| {
            current
                .checked_sub(bytes)
                .expect("external memory accounting underflow")
        });
    }
}

fn update_bytes(bytes: &AtomicUsize, update: impl Fn(usize) -> usize) {
    let mut current = bytes.load(Ordering::Acquire);
    loop {
        let next = update(current);
        match bytes.compare_exchange_weak(current, next, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => return,
            Err(actual) => current = actual,
        }
    }
}

/// An RAII account for memory owned by a host resource on behalf of a runtime.
///
/// Keep this value alongside the allocation it represents. Dropping the value
/// removes the allocation from [`RuntimeMemoryUsage::external_bytes`].
#[derive(Debug)]
#[must_use = "keep this value alive for as long as the attributed allocation exists"]
pub struct ExternalMemoryAllocation {
    tracker: ExternalMemoryTracker,
    bytes: usize,
}

impl ExternalMemoryAllocation {
    /// Return the number of bytes currently attributed to this allocation.
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    /// Update the attributed size after the host allocation changes.
    pub fn resize(&mut self, bytes: usize) {
        if bytes > self.bytes {
            self.tracker.add(bytes - self.bytes);
        } else {
            self.tracker.subtract(self.bytes - bytes);
        }
        self.bytes = bytes;
    }
}

impl Drop for ExternalMemoryAllocation {
    fn drop(&mut self) {
        self.tracker.subtract(self.bytes);
    }
}

/// A point-in-time, per-runtime memory report.
///
/// All contexts belonging to the same runtime are included because they share
/// one allocator, atom table, class table, and job queue.
///
/// `engine.malloc_size` is the memory reserved through the QuickJS allocator.
/// `external_bytes` contains host allocations explicitly attributed to this
/// runtime. These two values do not overlap, so [`Self::total_attributed_bytes`]
/// can add them. Other `engine` size fields are QuickJS's structural estimates
/// and must not be added to the total.
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct RuntimeMemoryUsage {
    /// The detailed report produced by `JS_ComputeMemoryUsage`.
    pub engine: qjs::JSMemoryUsage,
    /// Host-owned bytes registered against this runtime.
    pub external_bytes: usize,
    /// Whether `engine.malloc_size` includes allocator usable-size reporting.
    pub allocator_usable_size_available: bool,
}

impl RuntimeMemoryUsage {
    /// Bytes currently reserved through the QuickJS allocator.
    ///
    /// Check `allocator_usable_size_available` before treating this as
    /// a complete allocator total. The default allocator cannot report usable
    /// sizes on every target.
    pub fn allocator_bytes(&self) -> usize {
        self.engine.malloc_size.try_into().unwrap_or(0)
    }

    /// QuickJS allocator bytes plus explicitly attributed host allocations.
    pub fn total_attributed_bytes(&self) -> usize {
        self.allocator_bytes().saturating_add(self.external_bytes)
    }
}
