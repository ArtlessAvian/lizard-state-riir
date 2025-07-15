use lizstate_sequence_enumeration::IsSequenceable;
use lizstate_sequence_enumeration::SequenceOf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MyNonMaxU32;

impl IsSequenceable for MyNonMaxU32 {
    const MAX_EXCLUSIVE: u64 = u32::MAX as u64;

    fn to_value(self) -> u64 {
        Self::MAX_EXCLUSIVE - 1
    }

    fn from_value(_value: u64) -> Self {
        MyNonMaxU32
    }
}

#[test]
fn fits_two() {
    assert_eq!(SequenceOf::<MyNonMaxU32>::LARGEST_LEN, 2);

    let mut sequence = SequenceOf::<MyNonMaxU32>::new_empty();
    sequence.push(MyNonMaxU32).unwrap();
    sequence.push(MyNonMaxU32).unwrap();
    sequence.push(MyNonMaxU32).unwrap_err();
}
