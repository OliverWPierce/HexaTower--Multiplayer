/// This module provides the functionality for ensuring tile-related visuals remain consistent with the backend. This includes animations.
///
/// Indicators: I could have used a parent-child heirarchy between indicators and the tiles they indicate; however,
/// that creates a great deal of indirection to associate an indicator with the state of the tile it represents. Further,
/// storing a wrapper of a tile id uses less memory than storing two entity ids. Foregoing a parent-child relationship
/// between tiles and indicators comes at the cost of a) making tile movement more challenging, and b) clicking an indicator does
/// not bubble up to the tile.
use bevy::{color::palettes::tailwind::AMBER_700, prelude::*};
use core_game_logic::{
    pieces::FacingHexDirection,
    requests::ActionProcessCache,
    tile_based_actions::{self, SelectedTile},
    tile_mapping::*,
    tiles::TileType,
};

use crate::{
    functional_assets::{GameCreationSettings, LogicalWorld, SetUpBoard},
    inputs_interface::ActionInputManager,
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
        app.add_systems(Update, (swap_tile_mesh, roate_active_tile_visual));
        app.add_message::<TileTypeConverted>();

        app.add_systems(
            Update,
            (
                update_tile_selection_and_eligibility_indicators,
                manage_active_tile_visual,
            )
                .run_if(resource_changed::<ActionInputManager>),
        );

        app.add_observer(set_active_tile);
        app.add_observer(indicate_direction);
        app.add_observer(select_tile);
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
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    commands.spawn((
        Transform::default(),
        Mesh3d(asset_server.load("elligible_direction_indicator.obj")),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: AMBER_700.into(),
            emissive: LinearRgba::new(16.0, 14.0, 1.0, 1.0),
            ..default()
        })),
        DirectionIndicator,
        Visibility::Hidden,
        Pickable::IGNORE,
    ));

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

fn set_active_tile(
    mut click: On<Pointer<Click>>,
    vis_tiles: Query<&TileId>,
    vis_pieces: Query<&VisOccupies>,
    mut input_manager: ResMut<ActionInputManager>,
) {
    if let Some(cache) = input_manager.process_cache()
        && let ActionProcessCache::TileAction(..) = cache
    {
        return;
    }

    if let Ok(&tile) = vis_tiles.get(click.entity) {
        click.propagate(false);
        input_manager.set_active_tile(Some(tile));
    } else if let Ok(&VisOccupies(tile)) = vis_pieces.get(click.entity) {
        click.propagate(false);
        input_manager.set_active_tile(Some(tile));
    }
}

#[derive(Debug, Component)]
struct ActiveTileIndicator;

fn manage_active_tile_visual(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    alread_existing_indicators: Option<Single<(Entity, &mut Transform), With<ActiveTileIndicator>>>,
    active_tile: Res<ActionInputManager>,
) {
    if let Some(active_tile) = active_tile.active_tile() {
        let horizontal_location: Vec2 = HexVector2d::from(active_tile).into();

        if let Some(mut indicator) = alread_existing_indicators {
            let transform = &mut indicator.1;

            transform.translation.x = horizontal_location.x;
            transform.translation.z = horizontal_location.y;
        } else {
            commands.spawn((
                Transform::from_translation(Vec3 {
                    x: horizontal_location.x,
                    y: 1.0,
                    z: horizontal_location.y,
                }),
                ActiveTileIndicator,
                SceneRoot(
                    asset_server
                        .load(GltfAssetLabel::Scene(0).from_asset("active_tile_indicator.glb")),
                ),
                Pickable::IGNORE,
            ));
        }
    } else {
        if let Some(indicator) = alread_existing_indicators {
            commands.entity(indicator.0).despawn();
        }
    }
}

fn roate_active_tile_visual(
    mut indicator: Single<&mut Transform, With<ActiveTileIndicator>>,
    time: Res<Time>,
) {
    const SPEED: f32 = 1.0;
    indicator.rotate_y(SPEED * time.delta_secs());
}

#[derive(Debug, Component)]
struct DirectionIndicator;

fn indicate_direction(
    mut trigger: On<Pointer<Move>>,
    direction_indicator: Single<(&mut Transform, &mut Visibility), With<DirectionIndicator>>,
    tiles: Query<&TileId>,
    tile_meshes: Query<&ChildOf>,
    loaded_action: Res<ActionInputManager>,
) -> Result<(), BevyError> {
    let (mut transform, mut visibility) = direction_indicator.into_inner();

    if let Ok(&ChildOf(parent)) = tile_meshes.get(trigger.entity)
        && let Ok(tile) = tiles.get(parent)
        && let Some(ActionProcessCache::TileAction(selection_data)) = loaded_action.process_cache()
        && *selection_data.get_tile_state(*tile)?
            == core_game_logic::tile_based_actions::State::Elligible
        && let Some(target) = trigger.hit.position
    {
        trigger.propagate(false);
        *visibility = Visibility::Visible;

        let tile_position = Vec3::from(HexVector2d::from(*tile));

        transform.translation = Vec3::from(HexVector2d::from(*tile));
        transform.look_to(
            Vec3::from(HexVector2d::from(hex_direction_from_click_data(
                tile_position,
                target,
            ))),
            Dir3::Y,
        );

        Ok(())
    } else {
        *visibility = Visibility::Hidden;
        Ok(())
    }
}

fn hex_direction_from_click_data(tile_location: Vec3, hit_location: Vec3) -> FacingHexDirection {
    let hit_vector_with_tile_as_origin = hit_location.xz() - tile_location.xz();

    if hit_vector_with_tile_as_origin.y > 0.0 {
        if hit_vector_with_tile_as_origin.y < hit_vector_with_tile_as_origin.x * -SQRT_3 {
            FacingHexDirection::SouthEast
        } else if hit_vector_with_tile_as_origin.y < hit_vector_with_tile_as_origin.x * SQRT_3 {
            FacingHexDirection::SouthWest
        } else {
            FacingHexDirection::South
        }
    } else if hit_vector_with_tile_as_origin.y > hit_vector_with_tile_as_origin.x * -SQRT_3 {
        FacingHexDirection::NorthWest
    } else if hit_vector_with_tile_as_origin.y > hit_vector_with_tile_as_origin.x * SQRT_3 {
        FacingHexDirection::NorthEast
    } else {
        FacingHexDirection::North
    }
}

#[derive(Debug, Component, PartialEq, Clone, Copy)]
enum IndicatorType {
    Selected,
    Elligible,
}

#[derive(Debug, Component)]
struct IndicatorWatches(TileId);

fn update_tile_selection_and_eligibility_indicators(
    loaded_action: Res<ActionInputManager>,
    mut indicators: Query<(
        &mut Visibility,
        &mut Transform,
        &IndicatorType,
        &IndicatorWatches,
    )>,
) {
    if let Some(ActionProcessCache::TileAction(cache)) = loaded_action.process_cache() {
        let selection_states = cache.view_selection_states();

        for (mut visibility, mut transform, indicator_type, tile_watched) in indicators.iter_mut() {
            let Some(state) = selection_states.get(tile_watched.0.id() as usize) else {
                warn!("Tile indicator with invalid tile id {tile_watched:?}");
                continue;
            };

            match state {
                tile_based_actions::State::Elligible => {
                    if *indicator_type == IndicatorType::Elligible {
                        *visibility = Visibility::Visible
                    } else {
                        *visibility = Visibility::Hidden
                    }
                }
                tile_based_actions::State::Selected(facing_hex_direction) => {
                    if *indicator_type == IndicatorType::Selected {
                        transform.look_to(
                            Vec3::from(HexVector2d::from(*facing_hex_direction)),
                            Vec3::Y,
                        );
                        *visibility = Visibility::Visible;
                    } else {
                        *visibility = Visibility::Hidden;
                    }
                }
                tile_based_actions::State::Neither => *visibility = Visibility::Hidden,
            }
        }
    } else {
        for (mut visibility, ..) in indicators.iter_mut() {
            *visibility = Visibility::Hidden;
        }
    }
}

fn select_tile(
    mut trigger: On<Pointer<Click>>,
    tiles: Query<&TileId>,
    mut loaded_action: ResMut<ActionInputManager>,
    logical_world: Res<LogicalWorld>,
    direction_indicator: Single<&mut Visibility, With<DirectionIndicator>>,
) {
    let Ok(tile_id) = tiles.get(trigger.entity) else {
        return;
    };

    trigger.propagate(false);

    if let Some(hit_location) = trigger.hit.position
        && let Some(ActionProcessCache::TileAction(cache)) = loaded_action.process_cache_mut()
    {
        {
            let tile = SelectedTile {
                id: *tile_id,
                direction: hex_direction_from_click_data(
                    HexVector2d::from(*tile_id).into(),
                    hit_location,
                ),
            };

            if cache
                .try_select_tile_and_update_elligibility(tile, &logical_world.0)
                .is_ok()
            {
                *direction_indicator.into_inner() = Visibility::Hidden;
            }
        }
    }
}
