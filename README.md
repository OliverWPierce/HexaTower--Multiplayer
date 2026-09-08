This repository contains code for Hexatower: a chess inspired, turn-based, multiplayer, strategy game. I intend to release it on Steam later this year.

I'm making this repository public primarily for resume purposes. If you're interested in poking around the code, I've described two of my favorite parts below.

Copyright (c) [2026] [Oliver Pierce]. All rights reserved.

This repository and its content are made available solely for portfolio review and demonstration purposes. No permission is granted to copy, distribute, modify, or use this code in any project (commercial, academic, or personal) without explicit written permission.

## Hexagonal Coordinate System

**Go to core_game_logic/tile_mapping.rs**

For multiplayer to work, all the involved computers must have a consistent language for describing the game board. That language is TileIDs. Each tile on the board has a unique ID. This file contains the code for converting TileIds into cartesian coordinates and for relating tile IDs to each other. For example, this code allows me to ask "which tile is directly North of Tile 35?" and get an answer of "Tile 17." 

In addition to enabling multiplayer, this coordinate system improves performance by eliminating the need to traverse cached tile-adjacency data when calculating which moves the player can make. Instead of jumping around in memory, simple vector math is used.
## UI With Generics

**Go to hexatower_app/src/main_menu.rs** 

Look at the trait "ClickThroughSelector" and the generic function "display_clickthrough_selectors."

This trait enables me to easily setup UI elements for some game settings. It is practically free for performance and speeds up development time. Essentially, I define a new setting, choosing a previous and next value for each state of the setting. Then, I simply call "display_clickthrough_selector" for that setting, and a UI element which controls the setting appears. 

This function was challenging to create because I had to use phantom data to communicate with Bevy's systems. This also made it really fun to create.

## Interesting Files Related to Player Action Validation

core_game_logic/src/tile_based_actions/mod.rs
core_game_logic/src/tile_based_actions/selection_mechanics.rs
core_game_logic/src/requests.rs
hexatower_app/src/inputs_interface.rs

