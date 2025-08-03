use crate::custom_space::CustomSpace;
use crate::custom_space::Representative;
use crate::walk::reduced::Reduced;
use crate::walk::traits::IsAWalkMut;
use crate::walk::traits::IsAWalkRaw;

struct CustomSpaceBuilder {
    space: CustomSpace,
}

impl CustomSpaceBuilder {
    /// Adds tiles to create a dead end.
    ///
    /// Existing tiles will be reused.
    fn add_dead_end<Walk: IsAWalkMut>(&mut self, _from: Representative, _walk: Walk) {
        // Trivially, all new tiles along the path will be the shortest path.
        todo!()
    }

    /// Connect two adjacentish tiles.
    ///
    /// # Errors
    /// Tiles are not adjacentish.
    /// Tiles already have a connection in the direction between them.
    fn add_connection(&mut self, _from: Representative, _to: Representative) {
        todo!()
    }

    /// Adds tiles and a connection to form a circuit.
    ///
    /// The argument `circuit` has as many ups as downs, and as many rights as lefts.
    /// As a reduced walk in the free group, it will never self-intersect.
    fn add_circuit<Walk: IsAWalkRaw>(&mut self, _from: Representative, _circuit: Reduced<Walk>) {
        // split the walk into two halves. (the walk has even elements, and there are at least four.)
        // (let n = len() / 2, into_iter(), take(n).collect(), take(n).collect())

        // invert the second half. pop off an element.
        // assert that neither path immediately moves towards the origin.
        // add two dead ends for each half.
        // (there are now two paths to adjacentish tiles.)

        // add connection.

        // postcondition:
        // since the ends of both paths are equidistant from the origin,
        // all representatives made are the shortest.
        todo!()
    }

    // /// For each location in the space, choose new representatives by length.
    // ///
    // /// Ideally the builder will never let this be necessary. But it happens.
    // /// We can check if we need it by seeing if any key is much shorter/longer than its value
    // fn minimize(&mut self) {
    //     todo!()
    // }
}
