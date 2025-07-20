use lizstate_sequence_enumeration::digit::Digit;
use lizstate_sequence_enumeration::digit::IsSmallEnum;
use lizstate_sequence_enumeration::element_deque::DequeOf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MyU7;

impl IsSmallEnum for MyU7 {
    type Digit = Digit<{ 1 << 7 }>;

    fn to_digit(&self) -> Self::Digit {
        Digit::MAX
    }

    fn from_digit(_digit: Self::Digit) -> Self {
        MyU7
    }
}

#[test]
fn deque() {
    let mut yea = DequeOf::<MyU7, { 1 << 7 }, 9>::new_empty();
    yea.push_low(MyU7).unwrap();
    yea.push_low(MyU7).unwrap();
    yea.push_low(MyU7).unwrap();
    yea.push_low(MyU7).unwrap();
    yea.push_low(MyU7).unwrap();
    yea.push_low(MyU7).unwrap();
    yea.push_low(MyU7).unwrap();
    yea.push_low(MyU7).unwrap();
    yea.push_low(MyU7).unwrap();
    yea.push_low(MyU7).unwrap_err();
}
