use lizstate_sequence_enumeration::IsSequenceable;
use lizstate_sequence_enumeration::SequenceOf;
use lizstate_sequence_enumeration::ShiftSequenceOf;

// Please do not actually do this.
// This library won't stop you though.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Unary;

impl IsSequenceable for Unary {
    const MAX_EXCLUSIVE: u64 = 1;

    fn to_value(self) -> u64 {
        0
    }

    fn from_value(value: u64) -> Self {
        match value {
            0 => Unary,
            _ => unreachable!(),
        }
    }
}

#[test]
fn unary_representation() {
    let mut unary = SequenceOf::<Unary>::new_empty();
    for i in 0..10 {
        assert_eq!(unary.as_value(), i);
        assert_eq!(unary.len() as u64, i);
        unary.push(Unary).unwrap();
    }
    for i in (1..=10).rev() {
        assert_eq!(unary.as_value(), i);
        assert_eq!(unary.len() as u64, i);
        _ = unary.pop();
    }
    assert_eq!(unary.as_value(), 0);
    assert_eq!(unary.len(), 0);
}

#[test]
fn max_representation() {
    let mut unary = SequenceOf::<Unary>::from_value(SequenceOf::<Unary>::LARGEST);
    assert!(unary.is_full());

    _ = unary.pop();
    assert!(!unary.is_full());

    assert_eq!(unary.as_value(), 63);
}

#[test]
fn shift_fits_a_lot() {
    assert_eq!(ShiftSequenceOf::<Unary>::CAPACITY, 63);

    let mut sequence = ShiftSequenceOf::<Unary>::new_empty();
    for i in 0..63 {
        assert_eq!(sequence.len(), i);
        sequence.push_front(Unary).unwrap();
    }
    assert_eq!(sequence.len(), 63);
    dbg!(&sequence);

    sequence.push_front(Unary).unwrap_err();
    sequence.push_back(Unary).unwrap_err();
}
