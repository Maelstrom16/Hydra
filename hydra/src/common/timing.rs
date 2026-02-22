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

/// A counter that increases until a specific target value, then emits a single pulse.
pub struct DelayedTickCounter<T> {
    value: T,
    target: Option<T>
}

impl<T: Unsigned> DelayedTickCounter<T> {
    pub fn new(target: Option<T>) -> Self {
        DelayedTickCounter { value: T::ZERO, target }
    }

    pub fn count_to(&mut self, target: T) {
        self.value = T::ZERO;
        self.target = Some(target);
    }

    pub fn increment(&mut self) -> bool {
        if let Some(target) = self.target {
            self.value = (self.value + T::ONE) % target;
            if self.value == T::ZERO {
                self.target = None;
                return true;
            }
        }
        return false;
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

pub type OverflowCounter<T> = DynamicOverflowCounter<T, T>;
/// A counter that fires a pulse when overflowing.
/// 
/// Functions identically to a `DynamicModuloCounter`, if the
/// modulus were set to `T`'s max value.
pub struct DynamicOverflowCounter<T, R> {
    pub value: T,
    pub reset_value: R,
}

impl<T, R> DynamicOverflowCounter<T, R> where T: Unsigned, R: Borrow<T> + From<T> {
    pub fn new(starting_value: T) -> Self {
        DynamicOverflowCounter { value: starting_value, reset_value: T::ZERO.into()}
    }

    pub fn with_reset_value(starting_value: T, reset_value: R) -> Self {
        DynamicOverflowCounter { value: starting_value, reset_value }
    }

    /// Increments the counter, returning whether an overflow occurred.
    pub fn increment(&mut self) -> bool {
        let (new_value, overflow) = self.value.overflowing_add(T::ONE);
        self.value = match overflow {
            true => *self.reset_value.borrow(),
            false => new_value,
        };

        overflow
    }

    pub fn reset(&mut self) {
        self.value = *self.reset_value.borrow();
    }

    pub fn has_completed_cycle(&self) -> bool {
        self.value == *self.reset_value.borrow()
    }
}