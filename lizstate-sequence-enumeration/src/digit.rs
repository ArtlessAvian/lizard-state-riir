/// A number from 0..BASE
/// Digits are not `Clone` since we are using them
/// abstractly as colection elements.
#[derive(Debug, PartialEq, Eq, Hash)]
#[must_use]
pub struct Digit<const BASE: u64>(u64);

impl<const BASE: u64> Digit<BASE> {
    const _NO_UNARY_PLEASE: () = {
        assert!(BASE >= 2);
    };

    pub const ZERO: Self = Self::try_from(0).unwrap();
    pub const ONE: Self = Self::try_from(1).unwrap();
    pub const MAX: Self = Self::try_from(BASE - 1).unwrap();

    pub const fn try_from(value: u64) -> Option<Self> {
        if value < BASE {
            Some(Digit(value))
        } else {
            None
        }
    }

    pub const fn from_modulo(value: u64) -> Self {
        Self(value % BASE)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// For enums with few unique values.
///
/// If you implement this, you are probably also `Copy`.
///
/// There are other crates like Ordinalize or whatever.
pub trait IsSmallEnum {
    type Digit: Sized;

    fn to_digit(&self) -> Self::Digit;
    fn from_digit(digit: Self::Digit) -> Self;
}
