use crate::euclidean_plane::CartesianCoords;
use crate::euclidean_plane::EuclideanPlane;
use crate::euclidean_plane::impl_isagroup::PairwiseAddition;
use crate::free_group::FreeGroupElement;
use crate::free_group::impl_isagroup::FreeGroupConcat;
use crate::tiling::HasSquareTiling;
use crate::tiling::IsATile;

pub trait IsAGroup: Sized {
    type Element: IsATile;

    const IDENTITY: Self::Element;
    const UP: Self::Element;
    const RIGHT: Self::Element;

    #[must_use]
    fn inverse(&self, a: &Self::Element) -> Self::Element;

    /// A partial function from (Element x Element) -> (Element).
    #[must_use]
    fn op(&self, a: &Self::Element, b: &Self::Element) -> Option<Self::Element>;
}

///////////////////////////////////////////////////////////////////////////////////////////////////////

/// A function from a group to one of its subgroups.
///
/// The function must satisfy h(ab) = h(a)h(b). Notably,
/// a, b, and ab are elements of the domain.
/// h(a), h(b), and h(ab) are elements of the codomain.
trait GroupHomomorphism {
    type Domain: IsAGroup;
    type Codomain: IsAGroup;

    fn homomorphism(
        &self,
        from: <Self::Domain as IsAGroup>::Element,
    ) -> Option<<Self::Codomain as IsAGroup>::Element>;
}

struct FlattenFreeGroup;

impl GroupHomomorphism for FlattenFreeGroup {
    type Domain = FreeGroupConcat;
    type Codomain = PairwiseAddition;

    fn homomorphism(&self, free_group: FreeGroupElement) -> Option<CartesianCoords> {
        free_group
            .into_iter()
            .try_fold(PairwiseAddition::IDENTITY, |tile, dir| {
                EuclideanPlane.step(&tile, dir)
            })

        // It's helpful to think about all strings that map to the identity, aka the kernel.
        // That's strings with equal amounts of `Up`s and `Down`s, as well as `Right`s and `Left`s.

        // Strings of the form aba'b' are in the kernel.
        // Since h(aba'b') = h(e), that also means that h(ab) = h(ba).
        // And yep. PairwiseAddition is an abelian group.
    }
}
