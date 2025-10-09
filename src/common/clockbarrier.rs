// NOTE: Everything defined here in clockbarrier.rs
// is directly derived from the standard lib's barrier.rs

use std::fmt;
// FIXME(nonpoison_mutex,nonpoison_condvar): switch to nonpoison versions once they are available
use std::sync::{Condvar, Mutex};

/// A clock barrier enables multiple threads to synchronize the beginning
/// of some computation, while also providing access to its modulated generation ID.
/// 
/// Struct definition and all function implementations adapted from std::sync::Barrier.
pub struct ClockBarrier {
    lock: Mutex<ClockBarrierState>,
    cvar: Condvar,
    num_threads: usize,
    num_cycles: usize
}

// The inner state of a double barrier
struct ClockBarrierState {
    count: usize,
    generation_id: usize,
}

/// A `ClockBarrierWaitResult` is returned by [`ClockBarrier::wait()`] when all threads
/// in the [`ClockBarrier`] have rendezvoused.
///
/// Struct definition and all function implementations adapted from std::sync::BarrierWaitResult.
pub struct ClockBarrierWaitResult(bool);

impl fmt::Debug for ClockBarrier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClockBarrier").finish_non_exhaustive()
    }
}

impl ClockBarrier {
    #[must_use]
    #[inline]
    pub const fn new(t: usize, c: usize) -> ClockBarrier {
        ClockBarrier {
            lock: Mutex::new(ClockBarrierState { count: 0, generation_id: 0 }),
            cvar: Condvar::new(),
            num_threads: t,
            num_cycles: c
        }
    }

    pub fn wait(&self) -> ClockBarrierWaitResult {
        let mut lock = self.lock.lock().unwrap();
        let local_gen = lock.generation_id;
        lock.count += 1;
        if lock.count < self.num_threads {
            let _guard =
                self.cvar.wait_while(lock, |state| local_gen == state.generation_id).unwrap();
            ClockBarrierWaitResult(false)
        } else {
            lock.count = 0;
            lock.generation_id = lock.generation_id.wrapping_add(1) % self.num_cycles;
            self.cvar.notify_all();
            ClockBarrierWaitResult(true)
        }
    }

    #[inline]
    pub fn cycle(&self) -> usize {
        self.lock.lock().unwrap().generation_id
    }

    #[inline]
    pub fn new_frame(&self) -> bool {
        self.cycle() == 0
    }
}

impl fmt::Debug for ClockBarrierWaitResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClockBarrierWaitResult").field("is_leader", &self.is_leader()).finish()
    }
}

impl ClockBarrierWaitResult {
    #[must_use]
    pub fn is_leader(&self) -> bool {
        self.0
    }
}