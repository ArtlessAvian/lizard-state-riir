use lizstate_sequence_enumeration::IsSequenceable;
use lizstate_sequence_enumeration::SequenceOf;
use lizstate_sequence_enumeration::ShiftSequenceOf;
use lizstate_sequence_enumeration::digit::Digit;
use lizstate_sequence_enumeration::digit::IsSmallEnum;
use lizstate_sequence_enumeration::element_deque::DequeOf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MyNonMaxU8;

impl IsSequenceable for MyNonMaxU8 {
    const MAX_EXCLUSIVE: u64 = u8::MAX as u64;

    fn to_value(self) -> u64 {
        Self::MAX_EXCLUSIVE - 1
    }

    fn from_value(_value: u64) -> Self {
        MyNonMaxU8
    }
}

impl IsSmallEnum for MyNonMaxU8 {
    type Digit = Digit<{ u8::MAX as u64 }>;

    fn to_digit(&self) -> Self::Digit {
        Digit::MAX
    }

    fn from_digit(_digit: Self::Digit) -> Self {
        MyNonMaxU8
    }
}

#[test]
fn fits_eight() {
    assert_eq!(SequenceOf::<MyNonMaxU8>::LARGEST_LEN, 8);

    let mut sequence = SequenceOf::<MyNonMaxU8>::new_empty();
    for i in 0..8 {
        assert_eq!(sequence.len(), i);
        sequence.push(MyNonMaxU8).unwrap();
    }
    assert_eq!(sequence.len(), 8);
    dbg!(&sequence);

    sequence.push(MyNonMaxU8).unwrap_err();
}

#[test]
fn shift_fits_seven() {
    assert_eq!(ShiftSequenceOf::<MyNonMaxU8>::CAPACITY, 7);

    let mut sequence = ShiftSequenceOf::<MyNonMaxU8>::new_empty();
    for i in 0..7 {
        assert_eq!(sequence.len(), i);
        sequence.push_front(MyNonMaxU8).unwrap();
    }
    assert_eq!(sequence.len(), 7);
    dbg!(&sequence);

    sequence.push_front(MyNonMaxU8).unwrap_err();
    sequence.push_back(MyNonMaxU8).unwrap_err();
}

#[test]
fn deque() {
    let mut yea = DequeOf::<MyNonMaxU8, { u8::MAX as u64 }, 7>::new_empty();
    yea.push_low(MyNonMaxU8).unwrap();
    yea.push_low(MyNonMaxU8).unwrap();
    yea.push_low(MyNonMaxU8).unwrap();
    yea.push_low(MyNonMaxU8).unwrap();
    yea.push_low(MyNonMaxU8).unwrap();
    yea.push_low(MyNonMaxU8).unwrap();
    yea.push_low(MyNonMaxU8).unwrap();
    yea.push_low(MyNonMaxU8).unwrap_err();
}
