//! Efficiently stores a sequence of elements with very small representations.
//!
//! This uses enumeration in the math sense. Every unique sequence is mapped to a natural number.
//! However, we are still bounded by the size of representations.
//!
//! Since the sequence type represent collections, functions take mut references.
//! The provided types are `Copy` and their functions can be pure,
//! but you are unlikely to hold onto old collections.
//! Explicitly `Copy` if you do want that.
//!
//! # Possible extensions
//! This crate is very purpose made, otherwise I would be nerd sniping myself forever.
//!
//! ## Storage in other than u64.
//! It is possible to make this configurable to u32, u128, arrays of u64s, etc.
//! As long as it can represent a BigInt.
//!
//! You can already store 10+ digits of 4-ary in a u64, so more is not super necessary.
//!
//! ## Bijective function
//! A bijection between natural numbers and sequences exists, and can be described easily.
//! However, this isn't very useful.

#![no_std]

use core::hash::Hash;
use core::marker::PhantomData;

pub mod digit;

mod digit_deque;

pub mod element_deque;

/// Allows Sequence<Self> to be made.
pub trait IsSequenceable: Sized {
    /// Alternatively, the number of representations of Self.
    ///
    /// This should be 2 or more.
    /// (0 means Self is Never. 1 means Self is Unit.)
    const MAX_EXCLUSIVE: u64;

    /// All returned values *must* be in 0..MAX_EXCLUSIVE.
    fn to_value(self) -> u64;

    /// This must be the inverse of `IsSequenceable::to_u64`.
    fn from_value(value: u64) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceFull;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceEmpty;

/// Space efficient storage acting like a sequence of Elements.
///
/// # Guarantees
/// All sequences are represented with a unique natural number.
/// Given a shorter and longer sequence, the shorter's index is less than the longer's index.
/// All sequences of the same length are contiguous.
/// Has a constant time stack interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SequenceOf<Element: IsSequenceable>(u64, PhantomData<Element>);

// TODO: Create SequenceOf<Element, N: usize> to limit capacity.

impl<Element: IsSequenceable> SequenceOf<Element> {
    // Values lesser or equal are valid.
    pub const LARGEST: u64 = {
        let mut largest_of_len = 0u64;
        let mut len = 0;
        while len < 64
            && largest_of_len <= (u64::MAX - Element::MAX_EXCLUSIVE) / Element::MAX_EXCLUSIVE
        {
            largest_of_len *= Element::MAX_EXCLUSIVE;
            largest_of_len += Element::MAX_EXCLUSIVE;
            len += 1;
        }
        largest_of_len
    };

    /// The minimum of the largest len supported or 64.
    pub const LARGEST_LEN: usize = {
        let mut largest_of_len = 0u64;
        let mut len = 0;
        while len < 64
            && largest_of_len <= (u64::MAX - Element::MAX_EXCLUSIVE) / Element::MAX_EXCLUSIVE
        {
            largest_of_len *= Element::MAX_EXCLUSIVE;
            largest_of_len += Element::MAX_EXCLUSIVE;
            len += 1;
        }
        len
    };

    // Values greater or equal are full.
    const SMALLEST_FULL: u64 = {
        let largest_not_full = (Self::LARGEST - Element::MAX_EXCLUSIVE) / Element::MAX_EXCLUSIVE;
        largest_not_full + 1
    };

    pub const fn new_empty() -> Self {
        Self(0, PhantomData)
    }

    /// Ideally you shouldn't be able to do anything with this value.
    pub const fn as_value(&self) -> u64 {
        self.0
    }

    /// Very suspicious function. Hopefully you know what you are doing with this thing.
    pub const fn from_value(value: u64) -> Self {
        assert!(value <= Self::LARGEST);
        Self(value, PhantomData)
    }

    /// Whether the sequence can meaningfully pop an element.
    ///
    /// Since there is only one empty sequence, there is only one integer.
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_full(&self) -> bool {
        // Even if any element would fit, when self.0 * MAX_EXCLUSIVE + MAX_EXCLUSIVE <= u64::MAX
        // we don't want to accept it if a sequence of equal length *cannot* accept it.
        self.0 >= Self::SMALLEST_FULL
    }

    pub fn push(&mut self, el: Element) -> Result<(), SequenceFull> {
        if self.is_full() {
            Err(SequenceFull)
        } else {
            // Place the new element in the least significant digits.
            self.0 *= Element::MAX_EXCLUSIVE;
            self.0 += el.to_value() + 1; // 1..=MAX_EXCLUSIVE
            Ok(())
        }
    }

    pub fn peek(&self) -> Result<Element, SequenceEmpty> {
        if self.is_empty() {
            Err(SequenceEmpty)
        } else {
            // self.0 is in the form `n * MAX_EXCLUSIVE + r, where r in 1..=MAX_EXCLUSIVE`
            let r = (self.0 - 1) % Element::MAX_EXCLUSIVE;
            Ok(Element::from_value(r))
        }
    }

    pub fn pop(&mut self) -> Result<Element, SequenceEmpty> {
        if self.is_empty() {
            Err(SequenceEmpty)
        } else {
            let peek = self.peek();
            // self.0 is in the form `n * MAX_EXCLUSIVE + r, where r in 1..=MAX_EXCLUSIVE`
            let n = (self.0 - 1) / Element::MAX_EXCLUSIVE;
            self.0 = n;
            peek
        }
    }

    /// Returns the length.
    pub fn len(&self) -> usize {
        let mut value = self.0;

        for len in 0..=Self::LARGEST_LEN {
            if value == 0 {
                return len;
            } else {
                value -= 1;
                value /= Element::MAX_EXCLUSIVE;
            }
        }
        // The only way you get here is if you have a unary sequence and you intentionally mess with the internal values.
        unreachable!()
    }
}

/// Time efficient storage acting like a sequence of Elements.
///
/// # Guarantees
/// All sequences are represented with at least one natural number.
/// (Multiple natural numbers can represent the same sequence.)
/// Not all natural numbers represent valid sequences.
/// Has a constant time Deque interface.
/// Has a constant time index function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShiftSequenceOf<Element: IsSequenceable>(u64, PhantomData<Element>);

const fn ceil_ilog2(x: u64) -> u32 {
    if x == 0 {
        panic!()
    } else if x == 1 {
        0
    } else {
        (x - 1).ilog2() + 1
    }
}

impl<Element: IsSequenceable> ShiftSequenceOf<Element> {
    const BITS_PER_EL: u8 = {
        let option_el_max_exclusive = Element::MAX_EXCLUSIVE + 1;
        let bits = ceil_ilog2(option_el_max_exclusive);
        // bits is at most 64, which fits in a u8.
        bits as u8
    };

    pub const CAPACITY: u8 = { 63 / Self::BITS_PER_EL };

    const LOW_ORDER_MASK: u64 = {
        let mask_plus_one = 1 << Self::BITS_PER_EL;
        mask_plus_one - 1
    };

    pub const fn new_empty() -> Self {
        Self(1, PhantomData)
    }

    pub const fn is_empty(&self) -> bool {
        self.0 >> Self::BITS_PER_EL == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len() == Self::CAPACITY
    }

    pub const fn len(&self) -> u8 {
        // TODO: ilog2 or something.
        let mut len = 0;
        while len < Self::CAPACITY {
            let shift = self.0 >> (Self::BITS_PER_EL * len);
            let shift_past = shift >> Self::BITS_PER_EL;
            if shift_past == 0 {
                return len;
            }
            len += 1;
        }
        Self::CAPACITY
    }

    pub fn index(&self, i: u8) -> Option<Element> {
        if i >= self.len() {
            None
        } else {
            let shifted = self.0 >> (Self::BITS_PER_EL * i);
            let masked = shifted & Self::LOW_ORDER_MASK;
            Self::try_element_from_value(masked)
        }
    }

    pub fn push_front(&mut self, el: Element) -> Result<(), SequenceFull> {
        if self.is_full() {
            Err(SequenceFull)
        } else {
            self.0 <<= Self::BITS_PER_EL;
            self.0 += el.to_value();
            Ok(())
        }
    }

    pub fn peek_front(&self) -> Result<Element, SequenceEmpty> {
        self.index(0).ok_or(SequenceEmpty)
    }

    pub fn pop_front(&mut self) -> Result<Element, SequenceEmpty> {
        let peeked = self.peek_front()?;
        self.0 >>= Self::BITS_PER_EL;
        Ok(peeked)
    }

    pub fn push_back(&mut self, el: Element) -> Result<(), SequenceFull> {
        if self.is_full() {
            Err(SequenceFull)
        } else {
            let mask_out = Self::LOW_ORDER_MASK << (self.len() * Self::BITS_PER_EL);
            let mask = !mask_out;
            // Whatever junk is at the back, overwrite them with 0s.
            self.0 &= mask;
            // Since `self.0` has 0s where we want to write our value, we can use or instead of add.
            self.0 |= el.to_value() << (self.len() * Self::BITS_PER_EL);
            Ok(())
            // If you want to assume that the sequence properly ends in 0b000001,
            // you could probably do some xor shenanigans instead.
        }
    }

    pub fn peek_back(&self) -> Result<Element, SequenceEmpty> {
        if self.is_empty() {
            Err(SequenceEmpty)
        } else {
            Ok(self
                .index(self.len() - 1)
                .expect("len saw this was a valid element"))
        }
    }

    pub fn pop_back(&mut self) -> Result<Element, SequenceEmpty> {
        let peeked = self.peek_back()?;

        let len = self.len();
        let mask_out = Self::LOW_ORDER_MASK << ((len - 1) * Self::BITS_PER_EL);
        let mask = !mask_out;
        self.0 &= mask;
        self.0 |= 1 << ((len - 1) * Self::BITS_PER_EL);

        Ok(peeked)
    }

    pub fn try_element_from_value(value: u64) -> Option<Element> {
        if value >= Element::MAX_EXCLUSIVE {
            None
        } else {
            Some(Element::from_value(value))
        }
    }
}
