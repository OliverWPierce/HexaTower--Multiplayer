/// This module provides the functionality for ensuring tile-related visuals remain consistent with the backend. This includes animations.
///
/// Indicators: I could have used a parent-child heirarchy between indicators and the tiles they indicate; however,
/// that creates a great deal of indirection to associate an indicator with the state of the tile it represents. Further,
/// storing a wrapper of a tile id uses less memory than storing two entity ids. Foregoing a parent-child relationship
/// between tiles and indicators comes at the cost of a) making tile movement more challenging, and b) clicking an indicator does
/// not bubble up to the tile.
use bevy::prelude::*;
use core_game_logic::{
    tile_based_actions::{self, TileActionProcessCache},
    tile_mapping::*,
    tiles::TileType,
};

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
            BoardSize::Standard => 4,
            BoardSize::Large => 6,
            BoardSize::ExtraLarge => 8,
        }
    }
}

pub struct VisTilesPlugin;

impl Plugin for VisTilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, spawn_tiles_and_initialize_inficators);
        // switch this to a custom schedule later.
        app.add_systems(Update, swap_tile_mesh);
        app.add_message::<TileTypeConverted>();

        app.add_systems(
            Update,
            update_indicators.run_if(resource_changed_or_removed::<TileActionProcess>),
        );
    }
}

#[derive(Debug, Resource)]
struct VisualTileDirectory(Box<[Entity]>);
#[derive(Debug, Resource, Clone)]
struct TileModels {
    basic: Handle<Scene>,
    ex1: Handle<Scene>,
}

fn spawn_tiles_and_initialize_inficators(
    mut commands: Commands,
    settings: Res<GameCreationSettings>,
    asset_server: ResMut<AssetServer>,
) {
    let tile_models = TileModels {
        basic: asset_server.load(GltfAssetLabel::Scene(0).from_asset("tile_models/basic_tile.glb")),
        ex1: asset_server.load(GltfAssetLabel::Scene(0).from_asset("tile_models/example_tile.glb")),
    };

    let selected_indicator_mesh: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("selected_tile_indicator.glb"));
    let elligible_indicator_mesh: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("elligible_tile_indicator.glb"));

    commands.insert_resource(tile_models.clone());

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
                InheritedVisibility::VISIBLE,
                children![SceneRoot(tile_models.basic.clone()),],
            ))
            .id();

        vis_tiles.push(new_vis_tile);

        commands.spawn((
            IndicatorType::Elligible,
            Visibility::Hidden,
            Transform::from_translation(Vec3 {
                x: horizontal_location.x,
                y: 0.0,
                z: horizontal_location.y,
            }),
            SceneRoot(elligible_indicator_mesh.clone()),
            IndicatorWatches(tile_id),
        ));
        commands.spawn((
            IndicatorType::Selected,
            Visibility::Hidden,
            Transform::from_translation(Vec3 {
                x: horizontal_location.x,
                y: 0.0,
                z: horizontal_location.y,
            }),
            SceneRoot(selected_indicator_mesh.clone()),
            IndicatorWatches(tile_id),
        ));
    }

    commands.insert_resource(VisualTileDirectory(vis_tiles.into_boxed_slice()));
}

#[derive(Debug, Message)]
pub struct TileTypeConverted {
    pub tile: TileId,
    pub new_type: TileType,
}

fn swap_tile_mesh(
    mut change_information: MessageReader<TileTypeConverted>,
    parents_of_vis_tiles: Res<VisualTileDirectory>,
    mut commands: Commands,
    models: Res<TileModels>,
) -> Result<(), BevyError> {
    for event in change_information.read() {
        let parent = parents_of_vis_tiles
            .0
            .get(event.tile.id() as usize)
            .ok_or("No visual tile for this id")?;

        commands.entity(*parent).despawn_children();
        commands.spawn(SceneRoot(match event.new_type {
            TileType::Basic => models.basic.clone(),
            TileType::Ex1 => models.ex1.clone(),
        }));
    }

    Ok(())
}

#[derive(Debug, Resource)]
pub struct TileActionProcess(pub TileActionProcessCache);

#[derive(Debug, Component, PartialEq, Clone, Copy)]
enum IndicatorType {
    Selected,
    Elligible,
}

impl From<IndicatorType> for tile_based_actions::State {
    fn from(value: IndicatorType) -> Self {
        if value == IndicatorType::Selected {
            Self::Selected
        } else {
            Self::Elligible
        }
    }
}

#[derive(Debug, Component)]
struct IndicatorWatches(TileId);

fn update_indicators(
    maybe_selection_data: Option<Res<TileActionProcess>>,
    mut indicators: Query<(&mut Visibility, &IndicatorType, &IndicatorWatches)>,
) {
    let Some(selection_data) = maybe_selection_data else {
        for (mut visibility, ..) in indicators.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let selection_states = selection_data.0.view_selection_states();

    for (mut visibility, indicator_type, tile_watched) in indicators.iter_mut() {
        let Some(state) = selection_states.get(tile_watched.0.id() as usize) else {
            warn!("Tile indicator with invalid tile id {tile_watched:?}");
            continue;
        };

        if *state == (*indicator_type).into() {
            *visibility = Visibility::Visible
        } else {
            *visibility = Visibility::Hidden
        }
    }
}
