use lizstate_sequence_enumeration::digit::Digit;
use lizstate_sequence_enumeration::digit::IsSmallEnum;
use lizstate_sequence_enumeration::element_deque::PackedDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HexDigit {
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
    A,
    B,
    C,
    D,
    E,
    F,
}

impl IsSmallEnum for HexDigit {
    type Digit = Digit<16>;

    fn to_digit(&self) -> Self::Digit {
        match self {
            HexDigit::Zero => Digit::from_modulo(0),
            HexDigit::One => Digit::from_modulo(1),
            HexDigit::Two => Digit::from_modulo(2),
            HexDigit::Three => Digit::from_modulo(3),
            HexDigit::Four => Digit::from_modulo(4),
            HexDigit::Five => Digit::from_modulo(5),
            HexDigit::Six => Digit::from_modulo(6),
            HexDigit::Seven => Digit::from_modulo(7),
            HexDigit::Eight => Digit::from_modulo(8),
            HexDigit::Nine => Digit::from_modulo(9),
            HexDigit::A => Digit::from_modulo(0xA),
            HexDigit::B => Digit::from_modulo(0xB),
            HexDigit::C => Digit::from_modulo(0xC),
            HexDigit::D => Digit::from_modulo(0xD),
            HexDigit::E => Digit::from_modulo(0xE),
            HexDigit::F => Digit::from_modulo(0xF),
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
            0xA => Self::A,
            0xB => Self::B,
            0xC => Self::C,
            0xD => Self::D,
            0xE => Self::E,
            0xF => Self::F,
            (16..) => {
                unreachable!()
            }
        }
    }
}

#[test]
fn hex_representation() {
    let mut deque = PackedDeque::<HexDigit, 16, 15>::new_empty();

    deque.push_low(HexDigit::One).unwrap();
    deque.push_low(HexDigit::Three).unwrap();
    deque.push_low(HexDigit::Three).unwrap();
    deque.push_low(HexDigit::Seven).unwrap();

    assert_eq!(deque.get(), 0x1337 + 0x1111);
}

#[test]
fn fits_exactly_fifteen() {
    let mut deque = PackedDeque::<HexDigit, 16, 15>::new_empty();

    for _ in 0..15 {
        deque.push_low(HexDigit::F).unwrap();
    }

    assert_eq!(deque.get(), 0x0FFF_FFFF_FFFF_FFFF + 0x0111_1111_1111_1111);
}
