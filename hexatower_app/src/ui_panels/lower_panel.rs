use bevy::prelude::*;

use crate::{inputs_interface::LoadedAction, vis_tiles::ActiveTile};

fn manage_market_panel(active_tile: Option<Res<ActiveTile>>, action: Option<Res<LoadedAction>>) {}
