use std::ops::{Deref, DerefMut};

/// Util struct for representing a usize address, usize bank pair.
pub struct BankedAddress<A, B> {
    pub address: A,
    pub bank: B
}

/// Util struct for representing a 2D point or offset.
pub struct Coords<T> {
    pub x: T,
    pub y: T,
}

pub struct Delayed<T> {
    inner: T,
    queued: Option<T>
}

impl<T: Copy> Delayed<T> {
    pub fn get(&self) -> T {
        self.inner
    }

    pub fn queue(&mut self, val: T) {
        self.queued = Some(val)
    }

    pub fn process_queue(&mut self) {
        if let Some(val) = self.queued {
            self.inner = val;
            self.queued = None;
        }
    }

    pub fn cancel_queue(&mut self) {
        self.queued = None;
    }
}

impl<T> From<T> for Delayed<T> {
    fn from(value: T) -> Self {
        Self { inner: value, queued: None }
    }
}

impl<T> Deref for Delayed<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Delayed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}