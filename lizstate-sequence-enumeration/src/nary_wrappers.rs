use crate::digit::Digit;

/// An nary number leading with a 1 and then DIGITS other digits.
///
/// The value `\sum_i=0^{DIGITS} (a_i * b.pow(i))`, for digits a_n in base b.
///
/// Explicitly, thats:
/// a_0 * b.pow(0)
///     + a_1 * b.pow(1)
///     + a_2 * b.pow(2)
///     + a_3 * b.pow(3)
///     ...
///
/// TODO: Check if power of two BASE uses shifts and masks in MIR output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct LeadingOne<const BASE: u64, const DIGITS: u8>(u64);

impl<const BASE: u64, const DIGITS: u8> LeadingOne<BASE, DIGITS> {
    pub const ONE: Self = Self(1);

    pub const MAX: Self = {
        let mut max = 1;
        let mut i = 0;
        while i < DIGITS {
            max = max * BASE + (BASE - 1);
            i += 1;
        }
        Self(max)
    };

    pub const MIN_WITH_MSD: Self = {
        let mut min = 1;
        let mut i = 0;
        while i < DIGITS {
            min = min * BASE;
            i += 1;
        }
        Self(min)
    };

    pub const fn has_leading_one(value: u64) -> bool {
        let mut copy = value;
        loop {
            if copy == 1 {
                return true;
            }
            if copy < BASE {
                return false;
            }
            copy /= BASE;
        }
    }

    pub const fn try_from_value(value: u64) -> Option<Self> {
        if value <= Self::MAX.0 && Self::has_leading_one(value) {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn mul_base_add(self, digit: Digit<BASE>) -> Option<Self> {
        let digit = digit.get();
        if self.0 <= (Self::MAX.0 - digit) / BASE {
            Some(Self(self.0 * BASE + digit))
        } else {
            None
        }
    }

    pub const fn div_base(self) -> Option<Self> {
        if self.0 == 1 {
            None
        } else if self.0 >= BASE {
            Some(Self(self.0 / BASE))
        } else {
            unreachable!()
        }
    }

    pub const fn mod_base(self) -> Digit<BASE> {
        Digit::from_last_nary_digit(self.0)
    }

    pub const fn get_digit(self, mut i: u8) -> Digit<BASE> {
        let mut copy = self.0;
        while i > 0 {
            copy /= BASE;
            i -= 1;
        }
        Digit::from_last_nary_digit(copy)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::nary_wrappers::Digit;
    use crate::nary_wrappers::LeadingOne;

    #[test]
    fn consts() {
        assert_eq!(LeadingOne::<10, 4>::MAX.get(), 19999);
        assert_eq!(LeadingOne::<10, 3>::MAX.get(), 1999);
        assert_eq!(LeadingOne::<16, 4>::MAX.get(), 0x1_FFFF);

        assert_eq!(LeadingOne::<10, 4>::MIN_WITH_MSD.get(), 10000);
        assert_eq!(LeadingOne::<10, 3>::MIN_WITH_MSD.get(), 1000);
        assert_eq!(LeadingOne::<16, 4>::MIN_WITH_MSD.get(), 0x10000);
    }

    #[test]
    fn const_properties() {
        LeadingOne::<10, 4>::MIN_WITH_MSD
            .mul_base_add(Digit::ZERO)
            .ok_or(())
            .unwrap_err();
        LeadingOne::<10, 3>::MIN_WITH_MSD
            .mul_base_add(Digit::ZERO)
            .ok_or(())
            .unwrap_err();
        LeadingOne::<16, 4>::MIN_WITH_MSD
            .mul_base_add(Digit::ZERO)
            .ok_or(())
            .unwrap_err();
    }

    #[test]
    fn mul_base_add_digit() {
        let mut leet = LeadingOne::<10, 3>::ONE;
        leet = leet.mul_base_add(Digit::from_last_nary_digit(3)).unwrap();
        leet = leet.mul_base_add(Digit::from_last_nary_digit(3)).unwrap();
        leet = leet.mul_base_add(Digit::from_last_nary_digit(7)).unwrap();

        assert_eq!(leet.get(), 1337);

        leet.mul_base_add(Digit::from_last_nary_digit(0))
            .ok_or(())
            .unwrap_err();
    }

    #[test]
    fn divmod() {
        let mut leet = LeadingOne::<10, 3>::try_from_value(1337).unwrap();

        assert_eq!(leet.mod_base().get(), 7);
        leet = leet.div_base().unwrap();
        assert_eq!(leet.get(), 133);
        assert_eq!(leet.mod_base().get(), 3);
        leet = leet.div_base().unwrap();
        assert_eq!(leet.get(), 13);
        assert_eq!(leet.mod_base().get(), 3);
        leet = leet.div_base().unwrap();
        assert_eq!(leet.get(), 1);
        assert_eq!(leet.mod_base().get(), 1);
        assert!(leet.div_base().is_none());
    }
}
