use bevy::prelude::*;
use core_game_logic::{
    players::{InventoryIndex, PlayerId},
    requests::{ActionEffect, BackendRequest, InputData, RequestType, try_consume_request},
    tile_mapping::TileId,
};

use crate::{functional_assets::LogicalWorld, vis_tiles::TileTypeConverted};

pub struct InputInterfacePlugin;

impl Plugin for InputInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tmp_change_tile);
    }
}

fn write_message(effect: ActionEffect, commands: &mut Commands) {
    match effect {
        ActionEffect::ConvertedTileType { tile, new_type } => {
            commands.write_message(TileTypeConverted { tile, new_type });
        }
        _ => warn!(
            "Received an action effect from the logical world, but did not have a way to to display it to the player."
        ),
    }
}

fn tmp_change_tile(
    inputs: Res<ButtonInput<KeyCode>>,
    mut log_world: ResMut<LogicalWorld>,
    mut commands: Commands,
) {
    if !inputs.just_pressed(KeyCode::Space) {
        return;
    }

    let Ok(change_log) = try_consume_request(
        BackendRequest {
            acting_player: PlayerId(0),
            request: RequestType::UseCard {
                inventory_index: InventoryIndex(1),
                input: InputData::AffectedTiles(Box::new([TileId::new(0)])),
            },
        },
        &mut log_world.0,
    ) else {
        error!("could not make it happen.");
        return;
    };

    for effect in change_log.read() {
        write_message(effect.clone(), &mut commands);
    }
}
