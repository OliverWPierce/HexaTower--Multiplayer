/// This module provides the functionality for ensuring tile-related visuals remain consistent with the backend. This includes animations.
///
/// Indicators: I could have used a parent-child heirarchy between indicators and the tiles they indicate; however,
/// that creates a great deal of indirection to associate an indicator with the state of the tile it represents. Further,
/// storing a wrapper of a tile id uses less memory than storing two entity ids. Foregoing a parent-child relationship
/// between tiles and indicators comes at the cost of a) making tile movement more challenging, and b) clicking an indicator does
/// not bubble up to the tile.
use bevy::{asset::uuid::Error, math::FloatPow, prelude::*};
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
            remove_active_tile_indicators.run_if(resource_removed::<ActiveTile>),
        );

        app.add_observer(set_active_tile);
        app.add_observer(tmp_indicate_direction);
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
    let elligible_indicator_mesh: Handle<Scene> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("directional_elligible_tile_indicator.glb"));

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

        // {
        //     const ELLIGIBLE_INDICATOR_HEIGHT: f32 = 0.05;

        //     for direction in [
        //         FacingHexDirection::North,
        //         FacingHexDirection::South,
        //         FacingHexDirection::NorthEast,
        //         FacingHexDirection::SouthEast,
        //         FacingHexDirection::NorthWest,
        //         FacingHexDirection::SouthWest,
        //     ] {
        //         commands.spawn(
        //             (Transform::from_translation(Vec3 {
        //                 x: horizontal_location.x,
        //                 y: ELLIGIBLE_INDICATOR_HEIGHT,
        //                 z: horizontal_location.y,
        //             })
        //             .looking_to(Vec3::from(HexVector2d::from(direction)), Vec3::Y)),
        //         );
        //     }
        // }
    }

    commands.spawn((
        Transform::default(),
        SceneRoot(elligible_indicator_mesh.clone()),
        Tmp_Indicator,
    ));

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
struct Tmp_Indicator;

fn tmp_indicate_direction(
    taco: On<Pointer<Move>>,
    mut single: Single<&mut Transform, With<Tmp_Indicator>>,
    tiles: Query<&TileId>,
    meshes: Query<&ChildOf>,
) {
    if let Ok(&ChildOf(parent)) = meshes.get(taco.entity)
        && let Ok(tile) = tiles.get(parent)
        && let Some(target) = taco.hit.position
    {
        let tile_position = Vec2::from(HexVector2d::from(*tile));

        let hit_vector_with_tile_as_origin = target.xz() - tile_position;

        let hex_direction = {
            if hit_vector_with_tile_as_origin.y > 0.0 {
                if hit_vector_with_tile_as_origin.y < hit_vector_with_tile_as_origin.x * -SQRT_3 {
                    println!("north west");
                    SOUTH_EAST
                } else if hit_vector_with_tile_as_origin.y
                    < hit_vector_with_tile_as_origin.x * SQRT_3
                {
                    println!("north east");
                    SOUTH_WEST
                } else {
                    println!("north");
                    SOUTH
                }
            } else if hit_vector_with_tile_as_origin.y > hit_vector_with_tile_as_origin.x * -SQRT_3
            {
                NORTH_WEST
            } else if hit_vector_with_tile_as_origin.y > hit_vector_with_tile_as_origin.x * SQRT_3 {
                NORTH_EAST
            } else {
                println!("south");
                NORTH
            }
        };

        single.translation = Vec3::from(HexVector2d::from(*tile));
        single.look_to(Vec3::from(hex_direction), Dir3::Y);
    }
}
