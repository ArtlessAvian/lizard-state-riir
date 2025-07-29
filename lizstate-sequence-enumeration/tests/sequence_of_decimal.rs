use lizstate_sequence_enumeration::digit::Digit;
use lizstate_sequence_enumeration::digit::IsSmallEnum;
use lizstate_sequence_enumeration::element_deque::PackedDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecimalDigits {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl IsSmallEnum for DecimalDigits {
    type Digit = Digit<10>;

    fn to_digit(&self) -> Self::Digit {
        match self {
            DecimalDigits::Zero => Digit::from_modulo(0),
            DecimalDigits::One => Digit::from_modulo(1),
            DecimalDigits::Two => Digit::from_modulo(2),
            DecimalDigits::Three => Digit::from_modulo(3),
            DecimalDigits::Four => Digit::from_modulo(4),
            DecimalDigits::Five => Digit::from_modulo(5),
            DecimalDigits::Six => Digit::from_modulo(6),
            DecimalDigits::Seven => Digit::from_modulo(7),
            DecimalDigits::Eight => Digit::from_modulo(8),
            DecimalDigits::Nine => Digit::from_modulo(9),
        }
    }

    fn from_digit(digit: Self::Digit) -> Self {
        match digit.get() {
            0 => Self::Zero,
            1 => Self::One,
            2 => Self::Two,
            3 => Self::Three,
            4 => Self::Four,
            5 => Self::Five,
            6 => Self::Six,
            7 => Self::Seven,
            8 => Self::Eight,
            9 => Self::Nine,
            (10..) => {
                unreachable!()
            }
        }
    }
}

#[test]
fn decimal_representation() {
    let mut deque = PackedDeque::<DecimalDigits, 10, 19>::new_empty();

    deque.push_low(DecimalDigits::One).unwrap();
    deque.push_low(DecimalDigits::Three).unwrap();
    deque.push_low(DecimalDigits::Three).unwrap();
    deque.push_low(DecimalDigits::Seven).unwrap();

    assert_eq!(deque.get(), 1337 + 1111);
}
