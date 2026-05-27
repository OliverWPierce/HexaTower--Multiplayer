use bevy::prelude::*;
use core_game_logic::tile_mapping::*;

use crate::functional_assets::{GameCreationSettings, SetUpBoard};

#[derive(Debug)]
pub enum BoardSize {
    Small,
    Standard,
    Large,
    ExtraLarge,
}

impl BoardSize {
    fn into_ring_count(&self) -> u32 {
        match self {
            BoardSize::Small => 3,
            BoardSize::Standard => 5,
            BoardSize::Large => 6,
            BoardSize::ExtraLarge => 8,
        }
    }
}

pub struct VisTilesPlugin;

impl Plugin for VisTilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, sys_spawn_tiles);
    }
}

#[derive(Debug, Resource)]
struct VisualTileDirectory(Box<[Entity]>);
#[derive(Debug, Resource, Clone)]
struct TileModels {
    basic: Handle<Scene>,
}

fn sys_spawn_tiles(
    mut commands: Commands,
    settings: Res<GameCreationSettings>,
    asset_server: ResMut<AssetServer>,
) {
    let models = TileModels {
        basic: asset_server.load(GltfAssetLabel::Scene(0).from_asset("tile_models/basic_tile.glb")),
    };

    commands.insert_resource(models.clone());

    let rings_to_spawn = settings.board_size.into_ring_count();

    let mut vis_tiles = Vec::new();

    for tile_id in 0..core_game_logic::tile_mapping::tiles_on_board(rings_to_spawn) {
        let tile_id = TileId::new(tile_id);
        let horizontal_location: Vec2 = HexVector2d::from(tile_id).into();

        let new_vis_tile = commands
            .spawn((
                tile_id,
                Transform::from_translation(Vec3 {
                    x: horizontal_location.x,
                    y: 0.0,
                    z: horizontal_location.y,
                }),
                SceneRoot(models.basic.clone()),
            ))
            .id();

        vis_tiles.push(new_vis_tile);
    }

    commands.insert_resource(VisualTileDirectory(vis_tiles.into_boxed_slice()));
}
