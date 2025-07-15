use lizstate_sequence_enumeration::IsSequenceable;
use lizstate_sequence_enumeration::SequenceOf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MyU63 {
    Min,
    Max,
}

impl IsSequenceable for MyU63 {
    const MAX_EXCLUSIVE: u64 = u64::MAX / 2;

    fn to_value(self) -> u64 {
        match self {
            MyU63::Min => 0,
            MyU63::Max => Self::MAX_EXCLUSIVE - 1,
        }
    }

    fn from_value(_value: u64) -> Self {
        MyU63::Max
    }
}

#[test]
fn fits_after_min() {
    let mut sequence = SequenceOf::<MyU63>::new_empty();
    sequence.push(MyU63::Min).unwrap();
    sequence.push(MyU63::Max).unwrap();
    sequence.push(MyU63::Min).unwrap_err();
}

#[test]
fn doesnt_fit_after_max() {
    let mut sequence = SequenceOf::<MyU63>::new_empty();
    sequence.push(MyU63::Max).unwrap();
    sequence.push(MyU63::Min).unwrap_err();
}
