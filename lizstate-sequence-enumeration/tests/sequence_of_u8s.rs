use lizstate_sequence_enumeration::IsSequenceable;
use lizstate_sequence_enumeration::SequenceOf;
use lizstate_sequence_enumeration::ShiftSequenceOf;

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
fn shift_fits_eight() {
    assert_eq!(ShiftSequenceOf::<MyNonMaxU8>::CAPACITY, 8);

    let mut sequence = ShiftSequenceOf::<MyNonMaxU8>::new_empty();
    for i in 0..8 {
        assert_eq!(sequence.len(), i);
        sequence.push_front(MyNonMaxU8).unwrap();
    }
    assert_eq!(sequence.len(), 8);
    dbg!(&sequence);

    sequence.push_front(MyNonMaxU8).unwrap_err();
    sequence.push_back(MyNonMaxU8).unwrap_err();
}
