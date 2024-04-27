# lizard-state-riir
The name is facetious. This is a clone of the *logic* of my game in C# at https://github.com/artlessavian/lizard-state.
No graphics. No UI. Only organization and tests.

Learning Rust with this project, so the code may not be ideal/idiomatic.

(no jokes tho if the goal is to maximize my own enjoyment this would be the way to go. if the goal is to actually release the damn game then uhhhhhhhhh)

<small> (alternatively catch me writing the plot/dialogue on wattpad or something instead) </small>

## Benefits so far
* Floor is immutable, and updates are pure functions.
    * Floor is never in an intermediate state.
        * (This is just style. Making a mut clone is possible, but discouraged.)
        * If an operation breaks an invariant, no mutation has occured so we can just return Err. (Currently panics instead.)
    * Entities are immutable too.
        * Most entities do not change between turns, so cloning an Rc makes sense.
* Action verification
    * Actions are parsed into Commands. In the C# version, actions had a verification function that was called in UI and right before any mutation. There's a branch to approximate this behavior but its not sound.
    * In C#, actions were allowed to break "invariants," and systems were made to forcibly resolve them between turns.
* ~~Looser~~ No coupling to Godot
    * All game logic + data was a subclass of Resource.
    * C# needed to ask Godot to initialize Resources, so Godot could serialize them
    * Had trouble moving to 4.

## Current plans
* **Move all this into doc comments.**
* Action Targeting
    * Do you *need* to target anything other than a tile?
        * At tile corners, for aiming at the center of a 2x2? (Alt: offset selected tile)
        * At entities for tracking? (Alt: get entity from tile)
        * At nothing for convenience? (Alt: ignore tile)
* Opinionated code style???
    * i have no idea what im doing lol

## Observations
* My C# project depended a bit on interior mutability?
    * Mostly between model and view, the view would hold onto references to entities and sync with it between turns.
* Serialization means Rc's cannot be used internally.
    * (Depends on serialization library.)
    * This was previously handled by Godot references.
    * It is ok to copy Rc's between Floor generations.
* Orphan rule
    * oh man the orphan rule
    * Maybe the engine crate can import godot-ext to reduce one layer of newtype?
    * This also pretty much moves the *entire* glue crate into the engine, again because of the orphan rule. So no.