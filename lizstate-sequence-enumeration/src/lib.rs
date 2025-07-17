//! Efficiently stores a sequence of elements with very small representations.
//!
//! This uses enumeration in the math sense. Every natural number represents a unique sequence.
//!
//! Storage is hardcoded to u64.
//! It is possible to make this configurable to u32, u128, arrays of u64s, etc.
//! Eh.

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

/// Space efficient storage acting like a sequence of Elements.
///
/// # Guarantees
/// All sequences are represented with a unique natural number.
/// Given a shorter and longer sequence, the shorter's index is less than the longer's index.
/// All sequences of the same length are contiguous.
///
/// # Usage
/// Mutable interface acts like a stack with a maximum size.
/// While the type is `Copy`, the methods take `&mut self`
/// so you have to explicitly copy your sequence/collection.
///
/// # Internal Functionality
/// This is like how $n$-ary trees are represented contiguously in an array.
/// The children of a parent node $k$ is given through $nk + 1$ through $nk + n$.
/// Sequences also form a prefix tree! And since each element of the sequence is from the same set of n elements,
/// we get an $n$-ary tree.
///
/// If you're familiar with binary trees in arrays, they have nicer properties that we can't take advantage of.
/// Notably:
/// $$
///     \sum_{n=0}^N 2^n = 2^{N + 1} - 1
/// $$
/// TODO: This does generalize! For base b,
/// $$
///     \sum_{n=0}^N b^n = (1/(b-1)) * (b^{N + 1} - 1).
/// $$
/// While there is a division, this is a summation of integers, so the result must be an integer!
///
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
///
/// # Usage
/// Mutable interface acts like a `VecDeque` with a max size.
/// While the type is `Copy`, the methods take `&mut self`,
/// so you have to explicitly copy your sequence/collection.
///
/// # Internal Methodology.
/// TL;DR, this is an array of Option<Element>.
///
/// We have 64 bits. We want to take chunks of $B$ bits and interpret them as `Option<Element>`s.
/// $B$ bits represents values from $0..2**B$.
/// There are MAX_EXCLUSIVE values from 0..MAX_EXCLUSIVE that are like `Some`.
/// So, MAX_EXCLUSIVE..2**B values are like `None`.
///
/// An empty list should be all `None`s, but there's no obvious "nice" value.
/// `u64::MAX` to represent `0b11..1` works, but we could also use `-1i64`.
/// The bit patterns are the same, and multiplication and addition are the same, just "overflowing" at different times.
/// They both are equivalent to the ring of 2^64 elements, labeled differently.
///
/// Represent the empty sequence with -1.
/// $$ f([]) = -1 $$
/// Also, let $f(init + [last]) = f(init) * 2**B + last$.
/// The elements of the sequence are integers in 0..MAX_EXCLUSIVE.
/// Then,
/// $$
///     f([last]) = f([]) * 2**B + last
///               = -1 * 2**B + last
///     f([first, second]) = f([first]) * 2**B + second
///                        = (f([]) * 2**B + first) * 2**B + second
///                        = -1 * 2**2B + first * 2**B + second
/// $$
/// We see that elements are represented as powers of 2**B. And we can restore the values with div and mod.
/// Since 2**B is a power of 2, div and mod are just shifts and masking.
///
/// Note that shifting left always shifts 0s. Without overflow, this always represents multiplication by a power of two.
/// Shifting left overflows when the result's sign does not match the original.
/// If we encounter this, the storage is about full anyways.
///
/// There are multiple shifts right for signed and unsigned (nonnegative) types.
/// Signed types do aritmetic shifts. New bits match the sign bit. This is flooring division by a power of two.
/// Unsigned types do logical shifts. New bits are 0. This also is a flooring division for nonnegative numbers.
/// We want a flooring division for negative numbers, assuming positive numbers don't exist.
/// We can just OR 1's over the end. This is, ok, I guess. Just remember to do it!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShiftSequenceOf<Element: IsSequenceable>(i64, PhantomData<Element>);

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

    // We refuse to overwrite the sign bit.
    pub const CAPACITY: u8 = { 63 / Self::BITS_PER_EL };

    const LOW_ORDER_MASK: i64 = {
        let mask_plus_one = 1 << Self::BITS_PER_EL;
        mask_plus_one - 1
    };

    pub const fn new_empty() -> Self {
        Self(-1, PhantomData)
    }

    pub const fn is_empty(&self) -> bool {
        self.0 & Self::LOW_ORDER_MASK >= Element::MAX_EXCLUSIVE as i64
    }

    pub const fn is_full(&self) -> bool {
        self.len() == Self::CAPACITY
    }

    pub const fn len(&self) -> u8 {
        let mut len = 0;
        while len < Self::CAPACITY {
            let shifted = self.0 >> (Self::BITS_PER_EL * len);
            let masked = shifted & Self::LOW_ORDER_MASK;
            let masked = masked as u64;
            if masked >= Element::MAX_EXCLUSIVE {
                return len;
            }
            len += 1;
        }
        Self::CAPACITY
    }

    pub fn index(&self, i: u8) -> Option<Element> {
        if i < Self::CAPACITY {
            let shifted = self.0 >> (Self::BITS_PER_EL * i);
            let masked = shifted & Self::LOW_ORDER_MASK;
            let masked = masked as u64;
            Self::try_element_from_value(masked)
        } else {
            None
        }
    }

    pub fn push_front(&mut self, el: Element) -> Result<(), SequenceFull> {
        if self.is_full() {
            Err(SequenceFull)
        } else {
            self.0 = self.0.unbounded_shl(Self::BITS_PER_EL.into());
            self.0 += el.to_value() as i64;
            Ok(())
        }
    }

    pub fn peek_front(&self) -> Result<Element, SequenceEmpty> {
        self.index(0).ok_or(SequenceEmpty)
    }

    pub fn pop_front(&mut self) -> Result<Element, SequenceEmpty> {
        let peeked = self.peek_front()?;
        self.0 = self.0.unbounded_shr(Self::BITS_PER_EL as u32);
        self.0 |= Self::LOW_ORDER_MASK << (64 - Self::BITS_PER_EL);
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
            self.0 |= (el.to_value() as i64) << (self.len() * Self::BITS_PER_EL);
            Ok(())
            // If you want to assume that the sequence properly has all 1s,
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
        let write_ones = Self::LOW_ORDER_MASK << ((self.len() - 1) * Self::BITS_PER_EL);
        self.0 |= write_ones;
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
