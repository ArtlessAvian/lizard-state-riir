use lizstate_sequence_enumeration::IsSequenceable;
use lizstate_sequence_enumeration::SequenceOf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MyNonMaxU64;

impl IsSequenceable for MyNonMaxU64 {
    const MAX_EXCLUSIVE: u64 = u64::MAX;

    fn to_value(self) -> u64 {
        0
    }

    fn from_value(_value: u64) -> Self {
        MyNonMaxU64
    }
}

#[test]
fn fits_one() {
    assert_eq!(SequenceOf::<MyNonMaxU64>::LARGEST_LEN, 1);

    let mut sequence = SequenceOf::<MyNonMaxU64>::new_empty();
    sequence.push(MyNonMaxU64).unwrap();
    sequence.push(MyNonMaxU64).unwrap_err();
}
