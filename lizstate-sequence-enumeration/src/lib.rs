//! Efficiently stores a sequence of elements with very small representations.
//!
//! This uses enumeration in the math sense. Every natural number represents a unique sequence.

#![no_std]

use core::hash::Hash;
use core::marker::PhantomData;

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

/// Efficient storage acting like a sequence of Elements.
///
/// All sequences are represented with a unique natural number.
///
/// Shorter sequences are represented by smaller numbers.
/// All sequences of the same length are contiguous.
///
/// Mutable interface acts like a stack.
/// It's mutable despite being Copy, since you probably only want one sequence in scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SequenceOf<Element: IsSequenceable>(u64, PhantomData<Element>);

impl<Element: IsSequenceable> SequenceOf<Element> {
    pub fn new_empty() -> Self {
        Self(0, PhantomData)
    }

    /// Ideally you shouldn't be able to do anything with this value.
    pub fn as_value(&self) -> u64 {
        self.0
    }

    /// Very suspicious function. Hopefully you know what you are doing with this thing.
    pub fn from_value(value: u64) -> Self {
        Self(value, PhantomData)
    }

    /// Whether the sequence can meaningfully pop an element.
    ///
    /// Since there is only one empty sequence, there is only one integer.
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Whether this sequence can fit pushing *any* valid element.
    ///
    /// In rare cases where pushing *some* lower value may still fit, don't do that.
    /// In other rare cases, one sequence can fit, while another of the same length cannot.
    pub fn is_full(&self) -> bool {
        // self.0 * MAX_EXCLUSIVE + MAX_EXCLUSIVE > u64::MAX
        self.0 > (u64::MAX - Element::MAX_EXCLUSIVE) / Element::MAX_EXCLUSIVE
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

    /// Returns a value up to 64. Any sequence of greater length is not very useful.
    pub fn len(&self) -> usize {
        let mut value = self.0;

        for len in 0..64 {
            if value == 0 {
                return len;
            } else {
                value -= 1;
                value /= Element::MAX_EXCLUSIVE;
            }
        }
        // If you reached here, you are doing evil unary.
        64
    }
}
