use core::clone;
use core::marker::PhantomData;
use core::ops::Deref;

use crate::tiling::Tile;

/// A subset of tiles.
pub trait TileSet: 'static + Copy + Eq {
    type Element: Tile;

    fn contains(&self, el: &Self::Element) -> bool;

    fn try_prove(&self, el: &Self::Element) -> Option<ContainedIn<Self>> {
        self.contains(el).then_some(ContainedIn {
            set: *self,
            el: *el,
        })
    }

    /// Asserts the subset contains something, with a panic!
    ///
    /// # Panics
    /// The subset does not contain the element.
    fn assert_proof(&self, el: &Self::Element) -> ContainedIn<Self> {
        assert!(self.contains(el));
        ContainedIn {
            set: *self,
            el: *el,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ContainedIn<Set: TileSet> {
    set: Set,
    el: Set::Element,
}

impl<Set: TileSet> ContainedIn<Set> {
    pub fn as_set(&self) -> Set {
        self.set
    }
    pub fn as_el(&self) -> Set::Element {
        self.el
    }
}

impl<Set: TileSet> Tile for ContainedIn<Set> {}

///////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TileSetIntersection<A, B>(A, B);

impl<A, B> TileSet for TileSetIntersection<A, B>
where
    A: TileSet,
    B: TileSet<Element = A::Element>,
{
    type Element = A::Element;

    fn contains(&self, el: &Self::Element) -> bool {
        self.0.contains(el) && self.1.contains(el)
    }
}

/////////////////////////////////////////////////////////////////////

pub trait SubsetOf<Superset>
where
    Self: TileSet,
    Superset: TileSet<Element = Self::Element>,
{
    fn get_super(&self) -> Superset;

    /// Checks if a superset element is contained in the subset.
    fn subset_contains(&self, super_proof: &ContainedIn<Superset>) -> bool {
        self.contains(&super_proof.el)
    }

    /// Returns a proof a subset element is in the superset.
    fn superset_prove(proof: &ContainedIn<Self>) -> ContainedIn<Superset> {
        proof.as_set().get_super().assert_proof(&proof.as_el())
    }

    /// Returns a proof a superset element is in the subset (or not).
    fn subset_try_prove(&self, super_proof: &ContainedIn<Superset>) -> Option<ContainedIn<Self>> {
        self.subset_contains(super_proof).then_some(ContainedIn {
            set: *self,
            el: super_proof.as_el(),
        })
    }
}

impl<A, B> SubsetOf<A> for TileSetIntersection<A, B>
where
    Self: TileSet,
    A: TileSet<Element = Self::Element>,
    B: TileSet<Element = Self::Element>,
{
    fn get_super(&self) -> A {
        self.0
    }

    fn subset_contains(&self, super_proof: &ContainedIn<A>) -> bool {
        self.get_super() == super_proof.as_set() && self.1.contains(&super_proof.el)
    }
}

///////////////////////////////////////////////////////////

trait TileSetMapping: 'static + Copy + Eq {
    type From: TileSet;
    type To: TileSet;

    fn get_from(&self) -> Self::From;
    fn get_to(&self) -> Self::To;
    fn get_image(&self) -> Image<Self> {
        Image(*self)
    }

    // Given an element in `From`, return an element of `Image<Self>`, a subset of To.
    fn map(&self, proof_from: &ContainedIn<Self::From>) -> ContainedIn<Image<Self>>;

    // Given an element in `Image<Self>`, return an element of `From`.
    fn inverse(&self, proof_image: &ContainedIn<Image<Self>>) -> ContainedIn<Self::From>;

    // Given an element in `To`, a superset of `Image<Self>`, try to return an element of `From`.
    fn try_inverse(&self, proof_to: &ContainedIn<Self::To>) -> Option<ContainedIn<Self::From>>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Image<Mapping: TileSetMapping>(Mapping);

impl<Mapping> TileSet for Image<Mapping>
where
    Mapping: TileSetMapping,
{
    type Element = <Mapping::To as TileSet>::Element;

    fn contains(&self, el: &Self::Element) -> bool {
        self.0
            .get_to()
            .try_prove(el)
            .and_then(|proof| self.0.try_inverse(&proof))
            .is_some()
    }
}

impl<Mapping> SubsetOf<Mapping::To> for Image<Mapping>
where
    Mapping: TileSetMapping,
{
    fn get_super(&self) -> Mapping::To {
        self.0.get_to()
    }

    fn subset_contains(&self, super_proof: &ContainedIn<Mapping::To>) -> bool {
        self.0
            .get_to()
            .try_prove(&super_proof.as_el())
            .and_then(|proof| self.0.try_inverse(&proof))
            .is_some()
    }
}
