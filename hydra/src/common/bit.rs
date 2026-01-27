use std::{cell::Cell, marker::PhantomData, ops::{Deref, RangeInclusive, Shl}, rc::Rc};

use funty::Unsigned;

// TODO: Move GB specific behavior out

/// A type representing an unsigned collection of bits with read/write masks.
/// Generic type `U` represents the unsigned primitive type, and generic type
/// `I` is the type of a `DeserializedRegister<U>` into which this `MaskedBitSet`
/// may be deserialized into.
pub struct MaskedBitSet<T> {
    value: Cell<T>,
    read_mask: Cell<T>,
    write_mask: Cell<T>,
    write_fn: fn(&MaskedBitSet<T>, val: T),
}

impl<T> MaskedBitSet<T> where T: Unsigned {
    pub fn new(value: T, read_mask: T, write_mask: T, reg_type: WriteBehavior) -> Rc<MaskedBitSet<T>> {
        let write_fn = match reg_type {
            WriteBehavior::Standard => MaskedBitSet::write_standard,
            WriteBehavior::ResetOnWrite => MaskedBitSet::write_reset,
            WriteBehavior::UnmapBootRom => MaskedBitSet::write_boot,
        };

        Rc::new(MaskedBitSet { 
            value: Cell::new(value), 
            read_mask: Cell::new(read_mask), 
            write_mask: Cell::new(write_mask), 
            write_fn,
        })
    }

    pub fn new_unimplemented() -> Rc<MaskedBitSet<T>> {
        MaskedBitSet::new(T::ZERO.not(), T::ZERO, T::ZERO, WriteBehavior::Standard)
    }

    /// Returns a copy of the contained value.
    pub fn get(&self) -> T {
        self.value.get()
    }

    /// Sets the contained value.
    pub fn set(&self, val: T) {
        self.value.set(val)
    }

    /// Returns a copy of the contained value, with write-only
    /// and unimplemented bits replaced by a 1.
    pub fn read(&self) -> T {
        self.get() | !self.read_mask.get()
    }

    /// Sets the contained value, with read-only
    /// and unimplemented bits ignored.
    pub fn write(&self, val: T) {
        (self.write_fn)(self, val)
    }

    fn write_standard(&self, val: T) {
        let write_mask = self.write_mask.get();
        self.set((self.value.get() & !write_mask) | (val & write_mask))
    }

    fn write_reset(&self, _val: T) {
        self.set(T::ZERO);
    }

    fn write_boot(&self, _val: T) {
        todo!("BANK register write behavior is not yet implemented")
    }

    /// Redefines which bits of this register are
    /// readable and/or writable. 
    pub fn change_masks(&self, read_mask: T, write_mask: T) {
        self.read_mask.set(read_mask);
        self.write_mask.set(write_mask);
    }
}

pub enum WriteBehavior {
    Standard,
    ResetOnWrite,
    UnmapBootRom
}

/// A dummy struct used to represent a `MaskedBitSet`
/// that does not deserialize to any meaningful data.
pub struct UnimplementedBitSet {}
impl<T> FieldMap<T> for UnimplementedBitSet where T: Unsigned {
    fn deserialize(_value: T) -> Self {
        UnimplementedBitSet {}
    }
}

pub trait FieldMap<T: Sized> {
    fn deserialize(value: T) -> Self where Self: Sized;
}