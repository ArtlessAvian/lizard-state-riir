use crate::direction::Direction;
use crate::free_group::FreeGroupElement;
use crate::group::IsAGroup;

pub struct FreeGroupConcat;

impl IsAGroup for FreeGroupConcat {
    type Element = FreeGroupElement;

    const IDENTITY: Self::Element = FreeGroupElement::new();
    const UP: Self::Element = FreeGroupElement::new()
        .append_cancel(Direction::Up)
        .unwrap();
    const RIGHT: Self::Element = FreeGroupElement::new()
        .append_cancel(Direction::Right)
        .unwrap();

    fn inverse(&self, a: &Self::Element) -> Self::Element {
        FreeGroupElement(a.inverse())
    }

    fn op(&self, a: &Self::Element, b: &Self::Element) -> Option<Self::Element> {
        let mut ab = *a;
        for element in b.into_iter() {
            ab = ab.append_cancel(element)?;
        }
        Some(ab)
    }
}

#[cfg(test)]
mod tests {
    use crate::direction::Direction;
    use crate::free_group::FreeGroup;
    use crate::free_group::FreeGroupElement;
    use crate::free_group::impl_isagroup::FreeGroupConcat;
    use crate::group::IsAGroup;
    use crate::tiling::HasSquareTiling;

    #[test]
    fn group_basics() {
        for el in [
            FreeGroupConcat::IDENTITY,
            FreeGroupConcat::UP,
            FreeGroupConcat::RIGHT,
        ] {
            assert_eq!(
                FreeGroupConcat.op(&el, &FreeGroupConcat.inverse(&el)),
                Some(FreeGroupConcat::IDENTITY)
            );

            assert_eq!(
                FreeGroupConcat.op(&FreeGroupConcat.inverse(&el), &el),
                Some(FreeGroupConcat::IDENTITY)
            );
        }
    }

    #[test]
    fn group_inverse() {
        let path = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
            Direction::Left,
            Direction::Down,
            Direction::Right,
            Direction::Up,
        ];

        let string = FreeGroup.skip_path(&FreeGroupElement::new(), path).unwrap();

        assert_eq!(
            FreeGroupConcat.op(&string, &FreeGroupConcat.inverse(&string)),
            Some(FreeGroupConcat::IDENTITY)
        );

        assert_eq!(
            FreeGroupConcat.op(&FreeGroupConcat.inverse(&string), &string),
            Some(FreeGroupConcat::IDENTITY)
        );
    }
}
