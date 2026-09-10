# Hexatower

This repository contains code for Hexatower: a chess inspired, turn-based, multiplayer, strategy game. I intend to release it on Steam later this year. The multiplayer is built using a zero trust peer-to-peer listen-server architecture.

I'm making this repository public primarily for resume purposes. If you're interested in poking around the code, I've described two of my favorite parts below.

## Hexagonal Coordinate System

**Go to [core_game_logic/tile_mapping.rs](https://github.com/OliverWPierce/HexaTower--Multiplayer/blob/437d086ec887e750abfa4b6572289b0376d8aa16/core_game_logic/src/tile_mapping.rs)**

For multiplayer to work, all the involved computers must have a consistent language for describing the game board. That language is TileIDs. Each tile on the board has a unique ID. This file contains the code for converting TileIds into cartesian coordinates and for relating tile IDs to each other. For example, this code allows me to ask "which tile is directly North of Tile 35?" and get an answer of "Tile 17." 

In addition to enabling multiplayer, this coordinate system improves performance by eliminating the need to traverse cached tile-adjacency data when calculating which moves the player can make. Instead of jumping around in memory, simple vector math is used.
## UI With Generics

**Go to [hexatower_app/src/main_menu.rs](https://github.com/OliverWPierce/HexaTower--Multiplayer/blob/437d086ec887e750abfa4b6572289b0376d8aa16/hexatower_app/src/main_menu.rs)** 

Look at the trait "ClickThroughSelector" and the generic function "display_clickthrough_selectors."

This trait enables me to easily setup UI elements for some game settings. It is practically free for performance and speeds up development time. Essentially, I define a new setting, choosing a previous and next value for each state of the setting. Then, I simply call "display_clickthrough_selector" for that setting, and a UI element which controls the setting appears. 

This function was challenging to create because I had to use [PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html) (a zero-size struct which acts like it owns another type) to communicate with Bevy's systems. This also made it really fun to design.

## Interesting Files Related to Player Action Validation

1. [core_game_logic/src/tile_based_actions/mod.rs](https://github.com/OliverWPierce/HexaTower--Multiplayer/blob/437d086ec887e750abfa4b6572289b0376d8aa16/core_game_logic/src/tile_based_actions/mod.rs)
2. [core_game_logic/src/tile_based_actions/selection_mechanics.rs](https://github.com/OliverWPierce/HexaTower--Multiplayer/blob/437d086ec887e750abfa4b6572289b0376d8aa16/core_game_logic/src/tile_based_actions/selection_mechanics.rs)
3. [core_game_logic/src/requests.rs](https://github.com/OliverWPierce/HexaTower--Multiplayer/blob/437d086ec887e750abfa4b6572289b0376d8aa16/core_game_logic/src/requests.rs)
4. [hexatower_app/src/inputs_interface.rs](https://github.com/OliverWPierce/HexaTower--Multiplayer/blob/437d086ec887e750abfa4b6572289b0376d8aa16/hexatower_app/src/inputs_interface.rs)

