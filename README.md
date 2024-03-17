# lizard-state-riir
The name is facetious. This is a clone of the *logic* of my game in C# at https://github.com/artlessavian/lizard-state.
No graphics. No UI. Only organization and tests.

Learning Rust with this project, so the code may not be ideal/idiomatic.

(no jokes tho if the goal is to maximize my own enjoyment this would be the way to go. if the goal is to actually release the damn game then uhhhhhhhhh)

<small> (alternatively catch me writing the plot/dialogue on wattpad or something instead) </small>

## Benefits so far
* Floor is immutable, updates are pure functions.
    * Floor is never in an intermediate state.
        * If an operation breaks an invariant, no mutation has occured so we can just return Err. (Currently panics instead.)
    * Entities are immutable too.
        * Most entities do not change between turns, so cloning an Rc makes sense.
* Action verification
    * Actions are parsed into Commands. In the C# version, actions had a verification function that was called in UI and right before any mutation. There's a branch to approximate this behavior but its not sound.
    * In C#, actions were allowed to break "invariants," and systems were made to forcibly resolve them between turns.
* Looser coupling to Godot
    * Had trouble moving to 4.
    * Godot needed to initialize Resources to serialize them
    * Everything was a Godot Resource.

## Current plans
* Turntaking
    * Allow *set* of current turn takers instead of one at a time?
        * Player team first, then enemy "teams."
        * Player pass turn to partner?
    * Actions from previous command?
* I need to think about Actions vs Commands.
    * Actions are constants.
        * ~~Either unit type, constructable with locked down elements (eg enum Direction instead of struct Vector2i), or not publicly constructable.~~
        * Actions are aimed when parsing into Commands. (Action, Floor, Subject, T) -> Command
    * Do you *need* to target anything other than a tile?
        * At tile corners, for aiming at the center of a 2x2? (Alt: offset selected tile)
        * At entities for tracking? (Alt: get entity from tile)
        * At nothing for convenience? (Alt: ignore tile)
    * Commands are fallible?
        * If not, commands are proofs that an action's requirements are met (eg entity energy > 0) *and* the operation will not break an invariant or error?
    * QueuedActions with targets?
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
* Serialization means Rc's cannot be used internally.
    * This was previously handled by Godot references.
    * It is ok to copy Rc's between Floor generations.