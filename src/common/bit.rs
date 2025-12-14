use std::ops::{BitAnd, Shl};

pub trait BitSet {
    #[inline]
    fn bit(self, index: u8) -> bool where Self: Sized + Shl<u8, Output = Self> + BitAnd<Self, Output = Self> + PartialEq, u8: Into<Self>{
        self & (1.into() << index) != 0.into()
    }
}

impl BitSet for u8 {}