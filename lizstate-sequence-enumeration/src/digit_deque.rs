use crate::SequenceEmpty;
use crate::SequenceFull;
use crate::digit::Digit;

/// A bijection between digit sequences and the natural numbers.
/// The empty sequence is mapped to zero.
///
/// TODO: Explain the recurrence and how digits are extracted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct DigitDeque<const BASE: u64, const CAPACITY: u8>(u64);

impl<const BASE: u64, const CAPACITY: u8> DigitDeque<BASE, CAPACITY> {
    // Conveniently, this overflows when CAPACITY is too high.
    const MAX: Self = {
        let mut min = 0;
        let mut i = 0;
        while i < CAPACITY {
            min = min * BASE + BASE;
            i += 1;
        }
        Self(min)
    };

    pub const MIN_AT_CAPACITY: Self = {
        let pop_max = (Self::MAX.0 - 1) / BASE;
        Self(pop_max + 1)
    };

    pub const fn new_empty() -> Self {
        Self(0)
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn is_full(&self) -> bool {
        self.0 >= Self::MIN_AT_CAPACITY.0
    }

    pub const fn push_low(&mut self, digit: Digit<BASE>) -> Result<(), SequenceFull> {
        if self.is_full() {
            Err(SequenceFull)
        } else {
            self.0 *= BASE;
            self.0 += digit.get();
            self.0 += 1;
            Ok(())
        }
    }

    pub const fn peek_low(&self) -> Result<Digit<BASE>, SequenceEmpty> {
        if self.is_empty() {
            Err(SequenceEmpty)
        } else {
            Ok(Digit::from_modulo(self.0 - 1))
        }
    }

    pub const fn pop_low(&mut self) -> Result<(), SequenceEmpty> {
        if self.is_empty() {
            Err(SequenceEmpty)
        } else {
            self.0 -= 1;
            self.0 /= BASE;
            Ok(())
        }
    }

    pub const fn push_high(&mut self, digit: Digit<BASE>) -> Result<(), SequenceFull> {
        if self.is_full() {
            Err(SequenceFull)
        } else {
            let mut yeah = digit.get();
            yeah += 1;
            yeah *= BASE.pow(self.len() as u32);
            yeah += self.0;
            self.0 = yeah;
            Ok(())
        }
    }

    pub const fn peek_high(&self) -> Result<Digit<BASE>, SequenceEmpty> {
        if self.is_empty() {
            Err(SequenceEmpty)
        } else {
            let mut copy = self.0;
            while copy > BASE {
                copy /= BASE;
            }
            Ok(Digit::from_modulo(copy - 1))
        }
    }

    pub const fn pop_high(&mut self) -> Result<(), SequenceEmpty> {
        if self.is_empty() {
            Err(SequenceEmpty)
        } else {
            let mut power = 1;
            while self.0 > power * BASE {
                power *= BASE
            }
            self.0 %= power;
            Ok(())
        }
    }

    pub const fn len(&self) -> u8 {
        let mut out = 0;
        let mut copy = self.0;
        while copy > 0 {
            copy -= 1;
            copy /= BASE;
            out += 1;
        }
        out
    }

    pub const fn get(&self) -> u64 {
        self.0
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
    fn consts() {
        assert_eq!(DigitDeque::<10, 4>::MAX.0, 11110);
        assert_eq!(DigitDeque::<10, 3>::MAX.0, 1110);
        assert_eq!(DigitDeque::<16, 4>::MAX.0, 0x1_1110);

        assert_eq!(DigitDeque::<10, 4>::MIN_AT_CAPACITY.0, 1111);
        assert_eq!(DigitDeque::<10, 3>::MIN_AT_CAPACITY.0, 111);
        assert_eq!(DigitDeque::<16, 4>::MIN_AT_CAPACITY.0, 0x1111);
    }

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

    #[test]
    fn nines() {
        let mut deque = DigitDeque::<10, 4>::new_empty();

        deque.push_high(Digit::MAX).unwrap();
        deque.push_low(Digit::MAX).unwrap();
        deque.push_high(Digit::MAX).unwrap();
        deque.push_low(Digit::MAX).unwrap();
        deque.push_high(Digit::MAX).unwrap_err();
        deque.push_low(Digit::MAX).unwrap_err();

        deque.pop_high().unwrap();
        deque.pop_low().unwrap();
        deque.pop_high().unwrap();
        deque.pop_low().unwrap();
        deque.pop_high().unwrap_err();
        deque.pop_low().unwrap_err();
    }

    #[test]
    fn zeroes() {
        let mut deque = DigitDeque::<10, 4>::new_empty();

        deque.push_high(Digit::ZERO).unwrap();
        deque.push_low(Digit::ZERO).unwrap();
        deque.push_high(Digit::ZERO).unwrap();
        deque.push_low(Digit::ZERO).unwrap();
        deque.push_high(Digit::ZERO).unwrap_err();
        deque.push_low(Digit::ZERO).unwrap_err();

        deque.pop_high().unwrap();
        deque.pop_low().unwrap();
        deque.pop_high().unwrap();
        deque.pop_low().unwrap();
        deque.pop_high().unwrap_err();
        deque.pop_low().unwrap_err();
    }
}
