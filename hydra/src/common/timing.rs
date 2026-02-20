use std::{borrow::Borrow, ops::{Deref, DerefMut, Rem}};

use funty::Unsigned;

pub struct Resettable<T> {
    pub current: T,
    pub reset_value: T
}

impl<T: Copy> Resettable<T> {
    pub fn reset(&mut self) {
        self.current = self.reset_value;
    }
}

impl<T: Copy> From<T> for Resettable<T> {
    fn from(value: T) -> Self {
        Self { current: value, reset_value: value }
    }
}

impl<T> Borrow<T> for Resettable<T> {
    fn borrow(&self) -> &T {
        &self.current
    }
}

pub type ModuloCounter<T> = DynamicModuloCounter<T, T, T>;

/// A counter with preset overflow and reset values.
/// 
/// If `modulus == 0`, the counter will be disabled and stop incrementing when ticked.
pub struct DynamicModuloCounter<T, M, R> {
    pub value: T,
    pub modulus: M,
    pub reset_value: R,
}

impl<T, M, R> DynamicModuloCounter<T, M, R> where T: Unsigned, M: Borrow<T>, R: Borrow<T> + From<T> {
    pub fn new(starting_value: T, modulus: M) -> Self {
        DynamicModuloCounter { value: starting_value, reset_value: T::ZERO.into(), modulus }
    }

    pub fn with_reset_value(starting_value: T, modulus: M, reset_value: R) -> Self {
        DynamicModuloCounter { value: starting_value, modulus, reset_value }
    }

    /// Increments the counter, returning whether an overflow occurred.
    pub fn increment(&mut self) -> bool {
        let new_value = (self.value + T::ONE).checked_rem(*self.modulus.borrow());
        match new_value {
            Some(zero) if zero == T::ZERO => {
                self.value = *self.reset_value.borrow();
                return true;
            }
            Some(nonzero) => {
                self.value = nonzero;
                return false;
            }
            None => {
                return false;
            }
        }
    }

    pub fn reset(&mut self) {
        self.value = *self.reset_value.borrow();
    }

    pub fn has_completed_cycle(&self) -> bool {
        self.value == *self.reset_value.borrow()
    }
}