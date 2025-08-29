# Technical

Lizard State is a roguelike engine written in Rust. The user of the engine can define actions as a tree of subactions, and provide them to the player. The player can submit these actions and receive a tree of events. The game is displayed and animated in Godot.

## Meta

This document describes the technical requirements of the game. This explains how to interact with the engine. This also describes what should be possible in the game, though not all possibilities will be used.

This document is incomplete. This document does not describe an existing system. This document contains things that "would be cool" but unlikely to be useful.

# Crawler

The state of the game is immutable. Pure functions create copies of the gamestate and modify them. The user of the engine is allowed to hold onto old states of the game and rewind, if they would like.

## Topology
The topology of the game is an undirected graph. Every node is a 2D grid of tiles. Each grid has the same dimensions. Each node has a max degree of four, with a north, south, east, and west neighbor if they exist. The graph should be able to be "flattened" into a grid graph. You can draw lines, circles, "FOVs." and cones in the space.

Restrictions can be made at runtime. Two nodes may be connected to each other at runtime, provided they still can be "flattened." The space within each node are walls by default, and can be made into floors. Neighbors that do not exist are presumed to be all walls.

## Entities
Entities usually occupy space on the map. Each tile of the map has at most one occupier. Entities that occupy space can move. Entities have a team. Entities can hit other entities. Entities have health, and stop occupying space when they lose all health.

### Creatures
Creatures occupy one space. They can be temporarily knocked down and will stop occupying space until they get back up. They can take knockback and hitstun from attacks. Creatures can be blocking. If so, they take alternate knockback and hitstun.

### Snakes
Snakes always occupy a path of space. Snakes ignore knockback from attacks.

### Monsters
Monsters (TBD: name) always occupy a rectangle of space. Monsters ignore knockback and stun from attacks.

### Dropped Items
Dropped items do not occupy space, but are located somewhere. An entity can pick up items if they occupy their space.

### Projectiles
(Slow projectiles are unlikely to exist. All ranged attacks are "hitscan.")
Projectiles do not occupy space, but move along a defined path. They are removed at the end of their path. If they overlap any entity they hit them.

## Inventory
