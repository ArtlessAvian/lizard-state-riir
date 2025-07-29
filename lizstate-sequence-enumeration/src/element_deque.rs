use core::iter::Map;
use core::marker::PhantomData;

use crate::SequenceEmpty;
use crate::SequenceFull;
use crate::digit::Digit;
use crate::digit::IsSmallEnum;
use crate::digit_deque::DigitDeque;
use crate::digit_deque::LowToHighIter;

/// A deque of small enums, fitting inside 64 bits.
///
/// From an internal POV, this maps a sequence of digits to a sequence of Elements and back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct PackedDeque<Element, const BASE: u64, const CAPACITY: u8>(
    DigitDeque<BASE, CAPACITY>,
    PhantomData<Element>,
);

impl<Element, const BASE: u64, const CAPACITY: u8> PackedDeque<Element, BASE, CAPACITY>
where
    Element: IsSmallEnum<Digit = Digit<BASE>>,
{
    pub fn new_empty() -> Self {
        Self(DigitDeque::new_empty(), PhantomData)
    }

    pub fn push_low(&mut self, el: Element) -> Result<(), SequenceFull> {
        self.0.push_low(el.to_digit())
    }

    pub fn peek_low(&self) -> Result<Element, SequenceEmpty> {
        self.0.peek_low().map(Element::from_digit)
    }

    pub fn pop_low(&mut self) -> Result<(), SequenceEmpty> {
        self.0.pop_low()
    }

    pub fn push_high(&mut self, el: Element) -> Result<(), SequenceFull> {
        self.0.push_high(el.to_digit())
    }

    pub fn peek_high(&self) -> Result<Element, SequenceEmpty> {
        self.0.peek_high().map(Element::from_digit)
    }

    pub fn pop_high(&mut self) -> Result<(), SequenceEmpty> {
        self.0.pop_high()
    }

    pub fn len(&self) -> u8 {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get(&self) -> u64 {
        self.0.get()
    }
}

impl<Element, const BASE: u64, const CAPACITY: u8> IntoIterator
    for PackedDeque<Element, BASE, CAPACITY>
where
    Element: IsSmallEnum<Digit = Digit<BASE>>,
{
    type Item = Element;
    type IntoIter = Map<LowToHighIter<BASE, CAPACITY>, fn(Digit<BASE>) -> Element>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().map(Element::from_digit)
    }
}
