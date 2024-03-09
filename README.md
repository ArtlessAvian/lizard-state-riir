# lizard-state-2
The name is facetious. This is a clone of the *logic* of my game in C# at https://github.com/artlessavian/lizard-state.
No graphics. No UI. Only organization and tests.

Learning Rust with this project, so the code may not be ideal/idiomatic.

(no jokes tho if the goal is to maximize my own enjoyment this would be the way to go. if the goal is to actually release the damn game then uhhhhhhhhh)

## Benefits so far
* Floor is immutable, updates are pure functions.
    * Floor is never in an intermediate state.
        * If an operation breaks an invariant, no mutation has occured so we can just return Err.
    * Entities are immutable too.
        * Most entities do not change between turns, so cloning an Rc makes sense.
* Action verification
    * Actions are parsed into Commands. In the C# version, actions had a verification function that was called in UI and right before any mutation. There's a branch to approximate this behavior but its not sound.
    * In C#, actions were allowed to break "invariants" and systems would forcibly resolve them between turns.

## Current plans
* I need to think about Actions vs Commands.
    * Actions are constants? Either unit type, constructable with locked down elements (eg enum Direction instead of struct Vector2i), or not publicly constructable.
    * Commands are proofs that an action's requirements are met (eg entity energy > 0) *and* the operation will not break an invariant or error?
    * Currently there is no need for dyn ActionTrait, but I expect to use it. In C#, actions had a shared common interface meaning everything was aimed/configured with a (int, int) 2d vector. I could also shove every action into a big enum, but that's a bit silly.
* In ActionTrait.verify_action(), the entity and floor are not necessarily associated.
    * From floor, return a struct (Turntaker?) with the &self and &Rc<Entity>.
    * ActionTrait.verify_action takes Turntaker structs instead.
* Opt in to pure functions?
    * While most entities do not change over a turn, at least one entity must change.
    * All invariant checkers (formerly named Systems) must be cloned as well.
    * Rather than cloning a Floor, modifying the clone, and returning Ok(clone) or Err(), we can clone, modify &mut self, and restore from clone before erroring.
* Opinionated code style???
    * i have no idea what im doing lol

## Observations
* My C# project depended a bit on interior mutability?
    * Mostly between model and view, the view would hold onto references to entities and sync with it between turns.