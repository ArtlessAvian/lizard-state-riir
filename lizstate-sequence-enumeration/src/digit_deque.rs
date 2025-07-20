use crate::SequenceEmpty;
use crate::SequenceFull;
use crate::digit::Digit;
use crate::digit_sequence::LeadingOne;

/// An injection between sequences of Digits and the natural numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct DigitDeque<const BASE: u64, const CAPACITY: u8>(LeadingOne<BASE, CAPACITY>);

impl<const BASE: u64, const CAPACITY: u8> DigitDeque<BASE, CAPACITY> {
    pub const fn new_empty() -> Self {
        Self(LeadingOne::ONE)
    }

    pub const fn push_low(&mut self, digit: Digit<BASE>) -> Result<(), SequenceFull> {
        if let Some(some) = self.0.mul_base_add(digit) {
            self.0 = some;
            Ok(())
        } else {
            Err(SequenceFull)
        }
    }

    pub const fn peek_low(&self) -> Result<Digit<BASE>, SequenceEmpty> {
        if self.0.get() >= BASE {
            Ok(self.0.mod_base())
        } else {
            Err(SequenceEmpty)
        }
    }

    pub const fn pop_low(&mut self) -> Result<(), SequenceEmpty> {
        if self.is_empty() {
            Err(SequenceEmpty)
        } else if let Some(some) = self.0.div_base() {
            self.0 = some;
            Ok(())
        } else {
            Err(SequenceEmpty)
        }
    }

    pub const fn push_high(&mut self, digit: Digit<BASE>) -> Result<(), SequenceFull> {
        if self.is_full() {
            Err(SequenceFull)
        } else {
            let power = BASE.pow(self.len() as u32);

            let move_leading_one = self.0.get() - power + power * BASE;
            let set = move_leading_one + digit.get() * power;
            self.0 = LeadingOne::try_from_value(set).unwrap();

            Ok(())
        }
    }

    pub const fn peek_high(&self) -> Result<Digit<BASE>, SequenceEmpty> {
        if self.is_empty() {
            Err(SequenceEmpty)
        } else if self.0.get() <= BASE {
            unreachable!()
        } else {
            Ok(self.0.get_digit(self.len() - 1))
        }
    }

    pub const fn pop_high(&mut self) -> Result<(), SequenceEmpty> {
        if self.is_empty() {
            Err(SequenceEmpty)
        } else {
            let len = self.len();
            let old = self.0.get_digit(len - 1);
            let power = BASE.pow((len - 1) as u32);

            let zero_digit = self.0.get() - old.get() * power;
            let move_leading = zero_digit - power * BASE + power;

            self.0 = LeadingOne::try_from_value(move_leading).unwrap();
            Ok(())
        }
    }

    /// The number of `Digit`s in the deque.
    ///
    /// Alternatively, the power of the leading 1.
    pub const fn len(&self) -> u8 {
        let mut copy = *self;
        match copy.pop_low() {
            Ok(()) => 1 + copy.len(),
            Err(SequenceEmpty) => 0,
        }
    }

    pub const fn is_empty(&self) -> bool {
        if self.0.get() == 1 {
            true
        } else if self.0.get() < BASE {
            unreachable!()
        } else {
            false
        }
    }

    pub const fn is_full(&self) -> bool {
        self.0.get() >= LeadingOne::<BASE, CAPACITY>::MIN_WITH_MSD.get()
    }
}

pub struct LowToHighIter<const BASE: u64, const CAPACITY: u8>(DigitDeque<BASE, CAPACITY>);

impl<const BASE: u64, const CAPACITY: u8> IntoIterator for DigitDeque<BASE, CAPACITY> {
    type Item = Digit<BASE>;
    type IntoIter = LowToHighIter<BASE, CAPACITY>;

    fn into_iter(self) -> Self::IntoIter {
        LowToHighIter(self)
    }
}

impl<const BASE: u64, const CAPACITY: u8> Iterator for LowToHighIter<BASE, CAPACITY> {
    type Item = Digit<BASE>;

    fn next(&mut self) -> Option<Self::Item> {
        let out = self.0.peek_low().ok()?;
        _ = self.0.pop_low();
        Some(out)
    }
}

impl<const BASE: u64, const CAPACITY: u8> DoubleEndedIterator for LowToHighIter<BASE, CAPACITY> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let out = self.0.peek_high().ok()?;
        _ = self.0.pop_high();
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use crate::digit::Digit;
    use crate::digit_deque::DigitDeque;

    #[test]
    fn push_pop_low() {
        let mut deque = DigitDeque::<10, 4>::new_empty();

        for (len, digit) in (1..=4).zip([1, 3, 3, 7].into_iter()) {
            deque.push_low(Digit::try_from(digit).unwrap()).unwrap();
            assert_eq!(deque.peek_low().unwrap().get(), digit);
            assert_eq!(deque.len(), len);
        }

        deque.push_low(Digit::try_from(9).unwrap()).unwrap_err();

        for (len, digit) in (1..=4).zip([1, 3, 3, 7].into_iter()).rev() {
            assert_eq!(deque.peek_low().unwrap().get(), digit);
            assert_eq!(deque.len(), len);
            deque.pop_low().unwrap();
        }

        deque.peek_low().unwrap_err();
        deque.pop_low().unwrap_err();
    }

    #[test]
    fn push_pop_high() {
        let mut deque = DigitDeque::<10, 4>::new_empty();

        for (len, digit) in (1..=4).zip([1, 3, 3, 7].into_iter()) {
            deque.push_high(Digit::try_from(digit).unwrap()).unwrap();
            assert_eq!(deque.peek_high().unwrap().get(), digit);
            assert_eq!(deque.len(), len);
        }

        deque.push_high(Digit::try_from(9).unwrap()).unwrap_err();

        for (len, digit) in (1..=4).zip([1, 3, 3, 7].into_iter()).rev() {
            assert_eq!(deque.peek_high().unwrap().get(), digit);
            assert_eq!(deque.len(), len);
            deque.pop_high().unwrap();
        }

        deque.peek_high().unwrap_err();
        deque.pop_high().unwrap_err();
    }

    #[test]
    fn push_low_pop_high() {
        let mut deque = DigitDeque::<10, 4>::new_empty();

        for (len, digit) in (1..=4).zip([1, 3, 3, 7].into_iter()) {
            deque.push_low(Digit::try_from(digit).unwrap()).unwrap();
            assert_eq!(deque.peek_low().unwrap().get(), digit);
            assert_eq!(deque.len(), len);
        }

        deque.push_low(Digit::try_from(9).unwrap()).unwrap_err();

        for (len, digit) in (1..=4).rev().zip([1, 3, 3, 7].into_iter()) {
            assert_eq!(deque.peek_high().unwrap().get(), digit);
            assert_eq!(deque.len(), len);
            deque.pop_high().unwrap();
        }

        deque.peek_high().unwrap_err();
        deque.pop_high().unwrap_err();
    }

    #[test]
    fn push_high_pop_low() {
        let mut deque = DigitDeque::<10, 4>::new_empty();

        for (len, digit) in (1..=4).zip([1, 3, 3, 7].into_iter()) {
            deque.push_high(Digit::try_from(digit).unwrap()).unwrap();
            assert_eq!(deque.peek_high().unwrap().get(), digit);
            assert_eq!(deque.len(), len);
        }

        deque.push_high(Digit::try_from(9).unwrap()).unwrap_err();

        for (len, digit) in (1..=4).rev().zip([1, 3, 3, 7].into_iter()) {
            assert_eq!(deque.peek_low().unwrap().get(), digit);
            assert_eq!(deque.len(), len);
            deque.pop_low().unwrap();
        }

        deque.peek_low().unwrap_err();
        deque.pop_low().unwrap_err();
    }
}
