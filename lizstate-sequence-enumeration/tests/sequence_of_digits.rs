use lizstate_sequence_enumeration::IsSequenceable;
use lizstate_sequence_enumeration::SequenceOf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Digit {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten, // yeah ten is not a digit. if you want to prepend a zero, you want to subtract 1 and push a 10.
}

impl IsSequenceable for Digit {
    const MAX_EXCLUSIVE: u64 = 10;

    fn to_value(self) -> u64 {
        self as u64
    }

    fn from_value(value: u64) -> Self {
        match value {
            0 => Self::One,
            1 => Self::Two,
            2 => Self::Three,
            3 => Self::Four,
            4 => Self::Five,
            5 => Self::Six,
            6 => Self::Seven,
            7 => Self::Eight,
            8 => Self::Nine,
            9 => Self::Ten,
            (Self::MAX_EXCLUSIVE..) => {
                unreachable!()
            }
        }
    }
}

#[test]
fn decimal_representation() {
    assert_eq!(SequenceOf::<Digit>::LARGEST_LEN, 19);
    assert_eq!(SequenceOf::<Digit>::LARGEST, 11111111111111111110);
    // This seems like a weird number, before you remember that you can't push 0, but you can push 10.

    let mut number = 1337u64;
    let mut representation_backwards = SequenceOf::<Digit>::new_empty();
    while number > 0 {
        let digit = number % 10;
        number /= 10;

        representation_backwards
            .push(Digit::from_value(digit - 1))
            .unwrap();
    }

    assert_eq!(representation_backwards.as_value(), 7331)
}
