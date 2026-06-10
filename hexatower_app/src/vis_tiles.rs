/// This module provides the functionality for ensuring tile-related visuals remain consistent with the backend. This includes animations.
///
/// Indicators: I could have used a parent-child heirarchy between indicators and the tiles they indicate; however,
/// that creates a great deal of indirection to associate an indicator with the state of the tile it represents. Further,
/// storing a wrapper of a tile id uses less memory than storing two entity ids. Foregoing a parent-child relationship
/// between tiles and indicators comes at the cost of a) making tile movement more challenging, and b) clicking an indicator does
/// not bubble up to the tile.
use bevy::{asset::uuid::Error, prelude::*};
use core_game_logic::{
    pieces::FacingHexDirection,
    requests::{ActionProcessCache, RotationTileStates},
    tile_based_actions::{self},
    tile_mapping::*,
    tiles::TileType,
};

use crate::{
    functional_assets::{GameCreationSettings, LogicalWorld, SetUpBoard},
    inputs_interface::LoadedAction,
    vis_pieces::VisOccupies,
};

#[derive(Debug)]
pub enum BoardSize {
    Small,
    Standard,
    Large,
    ExtraLarge,
}

impl BoardSize {
    fn ring_count(&self) -> u32 {
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
        app.add_systems(Update, (swap_tile_mesh, visualize_active_tile));
        app.add_message::<TileTypeConverted>();

        app.add_systems(
            Update,
            (
                update_tile_selection_and_eligibility_indicators,
                manage_direction_indicators,
            )
                .run_if(resource_changed_or_removed::<LoadedAction>),
        );

        app.add_systems(
            Update,
            remove_active_tile_indicators.run_if(resource_removed::<ActiveTile>),
        );

        app.add_observer(tmp_select_tile);
        app.add_observer(set_active_tile);
        app.add_observer(choose_direction);
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

    let rings_to_spawn = settings.board_size.ring_count();

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
        commands.spawn((
            SceneRoot(match event.new_type {
                TileType::Basic => models.basic.clone(),
                TileType::Ex1 => models.ex1.clone(),
            }),
            ChildOf(*parent),
        ));
    }

    Ok(())
}

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

fn update_tile_selection_and_eligibility_indicators(
    loaded_action: Option<Res<LoadedAction>>,
    mut indicators: Query<(&mut Visibility, &IndicatorType, &IndicatorWatches)>,
) {
    if loaded_action.is_none() {
        for (mut visibility, _, _) in indicators.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        return;
    }

    let action = loaded_action.unwrap();

    match &action.cache {
        ActionProcessCache::TileAction(tile_action_process_cache) => {
            let selection_states = tile_action_process_cache.view_selection_states();

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
        ActionProcessCache::RotationAction { tile_data, .. } => match tile_data {
            RotationTileStates::ElligibleTiles(tile_ids) => {
                for (mut visibility, indicator_type, tile_watched) in indicators.iter_mut() {
                    if *indicator_type == IndicatorType::Elligible
                        && tile_ids.contains(&tile_watched.0)
                    {
                        *visibility = Visibility::Visible
                    } else {
                        *visibility = Visibility::Hidden
                    }
                }
            }
            RotationTileStates::SelectedTile(tile_id) => {
                for (mut visibility, indicator_type, tile_watched) in indicators.iter_mut() {
                    if tile_watched.0 != *tile_id || *indicator_type == IndicatorType::Elligible {
                        *visibility = Visibility::Hidden
                    } else {
                        *visibility = Visibility::Visible
                    }
                }
            }
        },
    }
}

fn tmp_select_tile(
    mut trigger: On<Pointer<Click>>,
    tiles: Query<&TileId>,
    mut loaded_action: If<ResMut<LoadedAction>>,
    logical_world: Res<LogicalWorld>,
) {
    let Ok(tile) = tiles.get(trigger.entity) else {
        return;
    };

    trigger.propagate(false);

    match &mut loaded_action.0.cache {
        ActionProcessCache::TileAction(tile_action_process_cache) => {
            let _ = tile_action_process_cache
                .try_select_tile_and_update_elligibility(*tile, &logical_world.0);
        }
        ActionProcessCache::RotationAction { tile_data, .. } => {
            if let RotationTileStates::ElligibleTiles(elligible) = &tile_data
                && elligible.contains(tile)
            {
                *tile_data = RotationTileStates::SelectedTile(*tile)
            }
        }
    }
}

#[derive(Debug, Resource)]
pub struct ActiveTile(pub TileId);

fn set_active_tile(
    mut click: On<Pointer<Click>>,
    vis_tiles: Query<&TileId>,
    vis_pieces: Query<&VisOccupies>,
    loaded_action: Option<Res<LoadedAction>>,
    mut commands: Commands,
) {
    if loaded_action.is_some() {
        return;
    }

    if let Ok(&tile) = vis_tiles.get(click.entity) {
        click.propagate(false);
        commands.insert_resource(ActiveTile(tile));
    } else if let Ok(&VisOccupies(tile)) = vis_pieces.get(click.entity) {
        click.propagate(false);
        commands.insert_resource(ActiveTile(tile));
    }
}

#[derive(Debug, Component)]
struct ActiveTileIndicator;

fn visualize_active_tile(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut alread_existing_indicators: Query<&mut Transform, With<ActiveTileIndicator>>,
    active_tile: If<Res<ActiveTile>>,
    time: Res<Time>,
) {
    let horizontal_location: Vec2 = HexVector2d::from(active_tile.0.0).into();

    if alread_existing_indicators.is_empty() {
        commands.spawn((
            Transform::from_translation(Vec3 {
                x: horizontal_location.x,
                y: 1.0,
                z: horizontal_location.y,
            }),
            ActiveTileIndicator,
            SceneRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset("active_tile_indicator.glb")),
            ),
        ));
    } else {
        const SPEED: f32 = 1.0;

        for mut ring in alread_existing_indicators.iter_mut() {
            ring.translation.x = horizontal_location.x;
            ring.translation.z = horizontal_location.y;

            ring.rotate_y(SPEED * time.delta_secs());
        }
    }
}

fn remove_active_tile_indicators(
    mut commands: Commands,
    indicators: Query<Entity, With<ActiveTileIndicator>>,
) {
    for entity in indicators {
        commands.entity(entity).despawn();
    }
}

#[derive(Debug, Component)]
struct DirectionIndicator(FacingHexDirection);

fn manage_direction_indicators(
    currently_existing_indicators: Query<Entity, With<DirectionIndicator>>,
    loaded_action: Option<Res<LoadedAction>>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
) {
    const HEIGHT: f32 = 0.4;

    let existing_indicator_count = currently_existing_indicators.iter().len();

    if let Some(loaded_action) = loaded_action
        && let ActionProcessCache::RotationAction {
            tile_data,
            selected_direction,
        } = &loaded_action.cache
        && let RotationTileStates::SelectedTile(tile_on_which_to_display) = tile_data
    {
        match selected_direction {
            Some(direction) => {
                if existing_indicator_count != 1 {
                    for entity in currently_existing_indicators.iter() {
                        commands.entity(entity).despawn();
                    }
                    commands.spawn((
                        Transform::from_translation(
                            Vec3::from(HexVector2d::from(*tile_on_which_to_display)).with_y(HEIGHT),
                        )
                        .looking_to(Vec3::from(HexVector2d::from(*direction)), Vec3::Y),
                        SceneRoot(asset_server.load(
                            GltfAssetLabel::Scene(0).from_asset("selected_direction_indicator.glb"),
                        )),
                        DirectionIndicator(*direction),
                    ));
                }
            }
            None => {
                if existing_indicator_count != 6 {
                    for entity in currently_existing_indicators.iter() {
                        commands.entity(entity).despawn();
                    }
                    let mesh: Handle<Scene> = asset_server.load(
                        GltfAssetLabel::Scene(0).from_asset("elligible_direction_indicator.glb"),
                    );
                    let hex_vec_location = HexVector2d::from(*tile_on_which_to_display);

                    commands.spawn((
                        Transform::from_translation(Vec3::from(hex_vec_location).with_y(HEIGHT))
                            .looking_to(Vec3::from(NORTH), Vec3::Y),
                        SceneRoot(mesh.clone()),
                        DirectionIndicator(FacingHexDirection::North),
                    ));
                    commands.spawn((
                        Transform::from_translation(Vec3::from(hex_vec_location).with_y(HEIGHT))
                            .looking_to(Vec3::from(NORTH_EAST), Vec3::Y),
                        SceneRoot(mesh.clone()),
                        DirectionIndicator(FacingHexDirection::NorthEast),
                    ));
                    commands.spawn((
                        Transform::from_translation(Vec3::from(hex_vec_location).with_y(HEIGHT))
                            .looking_to(Vec3::from(NORTH_WEST), Vec3::Y),
                        SceneRoot(mesh.clone()),
                        DirectionIndicator(FacingHexDirection::NorthWest),
                    ));
                    commands.spawn((
                        Transform::from_translation(Vec3::from(hex_vec_location).with_y(HEIGHT))
                            .looking_to(Vec3::from(SOUTH), Vec3::Y),
                        SceneRoot(mesh.clone()),
                        DirectionIndicator(FacingHexDirection::South),
                    ));
                    commands.spawn((
                        Transform::from_translation(Vec3::from(hex_vec_location).with_y(HEIGHT))
                            .looking_to(Vec3::from(SOUTH_EAST), Vec3::Y),
                        SceneRoot(mesh.clone()),
                        DirectionIndicator(FacingHexDirection::SouthEast),
                    ));
                    commands.spawn((
                        Transform::from_translation(Vec3::from(hex_vec_location).with_y(HEIGHT))
                            .looking_to(Vec3::from(SOUTH_WEST), Vec3::Y),
                        SceneRoot(mesh.clone()),
                        DirectionIndicator(FacingHexDirection::SouthWest),
                    ));
                }
            }
        }
    } else if existing_indicator_count != 0 {
        for entity in currently_existing_indicators.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn choose_direction(
    click: On<Pointer<Click>>,
    mut loaded_action: If<ResMut<LoadedAction>>,
    direction_indicators: Query<&DirectionIndicator>,
) {
    if let Ok(DirectionIndicator(direction)) = direction_indicators.get(click.entity)
        && let ActionProcessCache::RotationAction {
            selected_direction, ..
        } = &mut loaded_action.0.cache
    {
        *selected_direction = Some(*direction)
    }
}
