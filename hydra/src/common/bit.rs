use std::ops::{Deref, DerefMut};

use funty::{Integral, Unsigned};

#[macro_export]
macro_rules! bitmask_with_width {
    ($hi:literal..=$lo:literal) => {
        const{(1 << ($hi - $lo + 1)) - 1}
    };
}

#[macro_export]
macro_rules! let_or_set {
    ($field:ident, $($rest:tt)+) => {
        let $field = $($rest)+;
    };
    ($expr:expr, $($rest:tt)+) => {
        $expr = $($rest)+;
    };
}

#[macro_export]
macro_rules! serialize {
    () => {0};
    ($expr:expr =>> $hi:literal..=$lo:literal; $($rest:tt)*) => {
        (($expr & crate::bitmask_with_width!($hi..=$lo)) << $lo)
        | serialize!($($rest)*)
    };
    ($expr:tt => $hi:literal..=$lo:literal; $($rest:tt)*) => {
        ($expr & const{crate::bitmask_with_width!($hi..=$lo) << $lo})
        | serialize!($($rest)*)
    };
    ($expr:expr =>> $bit:literal; $($rest:tt)*) => {
        (($expr & 1) << $bit)
        | serialize!($($rest)*)
    };
    ($expr:tt => $bit:literal; $($rest:tt)*) => {
        ($expr & const{1 << $bit})
        | serialize!($($rest)*)
    };
    ($ones_mask:expr; $($rest:tt)*) => {
        ($ones_mask)
        | serialize!($($rest)*)
    };
}

#[macro_export]
macro_rules! deserialize {
    ($num:ident;) => {};
    ($num:ident; $hi:literal..=$lo:literal =>> $field:tt; $($rest:tt)*) => {
        crate::let_or_set!($field, ($num >> $lo) & crate::bitmask_with_width!($hi..=$lo));
        deserialize!($num; $($rest)*);
    };
    ($num:ident; $hi:literal..=$lo:literal => $field:tt; $($rest:tt)*) => {
        crate::let_or_set!($field, $num & const{crate::bitmask_with_width!($hi..=$lo) << $lo});
        deserialize!($num; $($rest)*);
    };
    ($num:ident; $bit:literal as bool =>> $field:tt; $($rest:tt)*) => {
        crate::let_or_set!($field, $num & const{1 << $bit} != 0);
        deserialize!($num; $($rest)*);
    };
    ($num:ident; $bit:literal =>> $field:tt; $($rest:tt)*) => {
        crate::let_or_set!($field, ($num >> $bit) & 1);
        deserialize!($num; $($rest)*);
    };
    ($num:ident; $bit:literal => $field:tt; $($rest:tt)*) => {
        crate::let_or_set!($field, $num & const{1 << $bit});
        deserialize!($num; $($rest)*);
    };
}

pub trait BitVec: Integral {
    fn test_bit(self, bit: Self) -> bool;
    fn test_bits(self, bitmask: Self) -> bool;
    fn set_bit(&mut self, bit: Self);
    fn set_bits(&mut self, bitmask: Self);
    fn reset_bit(&mut self, bit: Self);
    fn reset_bits(&mut self, bitmask: Self);
    fn map_bit(&mut self, bit: Self, cond: bool);
    fn map_bits(&mut self, bitmask: Self, cond: bool);
}

impl<T: Integral> BitVec for T {
    #[inline(always)]
    fn test_bit(self, bit: T) -> bool {
        self.test_bits(T::ONE << bit)
    }

    #[inline(always)]
    fn test_bits(self, bitmask: T) -> bool {
        self & bitmask != T::ZERO
    }

    #[inline(always)]
    fn set_bit(&mut self, bit: T) {
        self.set_bits(T::ONE << bit)
    }

    #[inline(always)]
    fn set_bits(&mut self, bitmask: T) {
        *self |= bitmask
    }

    #[inline(always)]
    fn reset_bit(&mut self, bit: T) {
        self.reset_bits(T::ONE << bit)
    }

    #[inline(always)]
    fn reset_bits(&mut self, bitmask: T) {
        *self &= !bitmask
    }

    #[inline(always)]
    fn map_bit(&mut self, bit: T, cond: bool) {
        match cond {
            true => self.set_bit(bit),
            false => self.reset_bit(bit),
        }
    }

    #[inline(always)]
    fn map_bits(&mut self, bitmask: T, cond: bool) {
        match cond {
            true => self.set_bits(bitmask),
            false => self.reset_bits(bitmask),
        }
    }
}

pub struct MaskedBitVec<T: BitVec, const MASKED_READ_VALUE: bool> {
    inner: T,
    read_mask: T,
    write_mask: T,
}

impl<T: BitVec, const MASKED_READ_VALUE: bool> MaskedBitVec<T, MASKED_READ_VALUE> {
    pub fn new(val: T, read_mask: T, write_mask: T) -> Self {
        MaskedBitVec { 
            inner: val,
            read_mask,
            write_mask
        }
    }

    #[inline(always)]
    pub fn test_bit(&self, bit: T) -> bool {
        self.inner.test_bit(bit)
    }

    #[inline(always)]
    pub fn test_bits(&self, bitmask: T) -> bool {
        self.inner.test_bits(bitmask)
    }

    #[inline(always)]
    pub fn set_bit(&mut self, bit: T) {
        self.inner.set_bit(bit)
    }

    #[inline(always)]
    pub fn set_bits(&mut self, bitmask: T) {
        self.inner.set_bits(bitmask)
    }

    #[inline(always)]
    pub fn reset_bit(&mut self, bit: T) {
        self.inner.reset_bit(bit)
    }

    #[inline(always)]
    pub fn reset_bits(&mut self, bitmask: T) {
        self.inner.reset_bits(bitmask)
    }

    #[inline(always)]
    pub fn map_bit(&mut self, bit: T, cond: bool) {
        self.inner.map_bit(bit, cond)
    }

    #[inline(always)]
    pub fn map_bits(&mut self, bitmask: T, cond: bool) {
        self.inner.map_bits(bitmask, cond)
    }

    #[inline(always)]
    pub fn write(&mut self, val: T) {
        self.inner = (self.inner & !self.write_mask) | (val & self.write_mask)
    }
}

impl<T: BitVec> MaskedBitVec<T, true> {
    #[inline(always)]
    pub fn read(&self) -> T {
        self.inner | !self.read_mask
    }
}

impl<T: BitVec> MaskedBitVec<T, false> {
    #[inline(always)]
    pub fn read(&self) -> T {
        self.inner & self.read_mask
    }
}

impl<T: BitVec, const MASK_VALUE: bool> Deref for MaskedBitVec<T, MASK_VALUE> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: BitVec, const MASK_VALUE: bool> DerefMut for MaskedBitVec<T, MASK_VALUE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}