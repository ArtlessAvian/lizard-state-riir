use crate::euclidean_plane::CartesianCoords;
use crate::euclidean_plane::EuclideanPlane;
use crate::euclidean_plane::PairwiseAddition;
use crate::free_group::FreeGroupConcat;
use crate::free_group::FreeGroupElement;
use crate::tiling::HasSquareTiling;
use crate::tiling::Tile;

pub trait GroupOp: Sized {
    type Element: Tile;

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
/// The function must satisfy h(ab) = h(a)h(b).
trait GroupHomomorphism {
    type Domain: GroupOp;
    type Codomain: GroupOp;

    /// The function.
    fn homomorphism(
        &self,
        from: <Self::Domain as GroupOp>::Element,
    ) -> Option<<Self::Codomain as GroupOp>::Element>;
}

struct FlattenFreeGroup;

impl GroupHomomorphism for FlattenFreeGroup {
    type Domain = FreeGroupConcat;
    type Codomain = PairwiseAddition;

    fn homomorphism(&self, free_group: FreeGroupElement) -> Option<CartesianCoords> {
        free_group
            .iter()
            .try_fold(PairwiseAddition::IDENTITY, |tile, dir| {
                EuclideanPlane.go(&tile, dir)
            })

        // It's helpful to think about all strings that map to the identity, aka the kernel.
        // That's strings with equal amounts of `Up`s and `Down`s, as well as `Right`s and `Left`s.

        // Strings of the form aba'b' are in the kernel.
        // Since h(aba'b') = h(e), that also means that h(ab) = h(ba).
        // And yep. PairwiseAddition is an abelian group.
    }
}
