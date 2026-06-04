use std::f32::consts::PI;

use bevy::prelude::*;
use core_game_logic::{
    pieces::{ArchetypeId, FacingHexDirection},
    players::PlayerId,
    tile_mapping::{HexVector2d, TileId},
};

use crate::vis_pieces::visual_piece_archetypes_storage::{
    BasePlatesDirectory, VisualPieceArchetypeDirectory,
};

pub struct VisPiecesPlugin;

impl Plugin for VisPiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PieceSpawned>();
        app.add_message::<StartFreeRotationAnimation>();
        app.add_message::<EndFreeRotationAnimation>();
        app.add_systems(Update, (spawn_visuals, start_free_rotation_animation));
    }
}

pub mod visual_piece_archetypes_storage {

    use bevy::prelude::*;
    use core_game_logic::{pieces::ArchetypeId, players::PlayerId};
    use thiserror::Error;

    #[derive(Debug, Resource)]
    pub struct VisualPieceArchetypeDirectory(Box<[VisualPieceArchetype]>);

    #[derive(Debug, Error)]
    #[error(
        "Could not find frontend display information for piece with archetype id: {0:?}. This includes its model, name, ect."
    )]
    pub struct MissingVisualDetails(ArchetypeId);

    impl VisualPieceArchetypeDirectory {
        pub fn get_visual_details(
            &self,
            id: ArchetypeId,
        ) -> Result<&VisualPieceArchetype, MissingVisualDetails> {
            self.0.get(id.0 as usize).ok_or(MissingVisualDetails(id))
        }

        pub fn new(archetypes: &[VisualPieceArchetype]) -> Self {
            Self(archetypes.into())
        }
    }
    #[derive(Debug, Clone)]
    pub struct VisualPieceArchetype {
        pub model: Handle<Scene>,
        pub name: String,
    }

    #[derive(Debug, Resource)]
    pub struct BasePlatesDirectory(Box<[Handle<Scene>]>);

    #[derive(Debug, Error)]
    #[error("Could not find a baseplate model for player {0:?}.")]
    pub struct MissingBaseplate(PlayerId);

    impl BasePlatesDirectory {
        pub fn get_base_plate(&self, id: PlayerId) -> Result<&Handle<Scene>, MissingBaseplate> {
            self.0.get(id.0 as usize).ok_or(MissingBaseplate(id))
        }

        pub fn new(models: &[Handle<Scene>]) -> Self {
            Self(models.into())
        }
    }
}

#[derive(Debug, Message)]
pub struct PieceSpawned {
    pub tile: TileId,
    pub archetype: ArchetypeId,
    pub owner: PlayerId,
    pub direction: FacingHexDirection,
}
#[derive(Debug, Component)]
struct PieceOnTile(TileId);

const BASEPLATE_HEIGHT: f32 = 0.115;

fn rotation_from_hex_direction(direction: &FacingHexDirection) -> Quat {
    Quat::from_axis_angle(
        Vec3::Y,
        match direction {
            FacingHexDirection::NorthEast => PI / 6.0,
            FacingHexDirection::North => PI / 2.0,
            FacingHexDirection::NorthWest => 5.0 * PI / 6.0,
            FacingHexDirection::SouthWest => 7.0 * PI / 6.0,
            FacingHexDirection::South => 3.0 * PI / 2.0,
            FacingHexDirection::SouthEast => 11.0 * PI / 6.0,
        } + PI / 2.0,
    )
}

fn spawn_visuals(
    mut spawn_events: MessageReader<PieceSpawned>,
    mut commands: Commands,
    base_plates: Res<BasePlatesDirectory>,
    piece_details: Res<VisualPieceArchetypeDirectory>,
) -> Result<(), BevyError> {
    for spawn in spawn_events.read() {
        let horizontal_location: Vec2 =
            core_game_logic::tile_mapping::HexVector2d::from(spawn.tile).into();
        commands.spawn((
            PieceOnTile(spawn.tile),
            Transform::from_translation(Vec3 {
                x: horizontal_location.x,
                y: 0.0,
                z: horizontal_location.y,
            })
            .with_rotation(rotation_from_hex_direction(&spawn.direction)),
            SceneRoot(base_plates.get_base_plate(spawn.owner)?.clone()),
            children![
                SceneRoot(
                    piece_details
                        .get_visual_details(spawn.archetype)?
                        .model
                        .clone()
                ),
                Transform::from_translation(Vec3 {
                    x: 0.0,
                    y: BASEPLATE_HEIGHT,
                    z: 0.0
                })
            ],
        ));
    }

    Ok(())
}

#[derive(Debug, Message)]
pub struct StartFreeRotationAnimation {
    pub on_tile: TileId,
}

#[derive(Debug, Message)]
pub struct EndFreeRotationAnimation {
    pub on_tile: TileId,
}

fn start_free_rotation_animation(
    mut pieces_to_start: MessageReader<StartFreeRotationAnimation>,
    asset_server: ResMut<AssetServer>,
    mut commands: Commands,
) {
    for StartFreeRotationAnimation { on_tile } in pieces_to_start.read() {
        let horizontal_location: Vec2 =
            core_game_logic::tile_mapping::HexVector2d::from(*on_tile).into();
        commands.spawn((
            Transform::from_translation(Vec3 {
                x: horizontal_location.x,
                y: 0.0,
                z: horizontal_location.y,
            }),
            SceneRoot(
                asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("free_rotation_indication_pillar.glb"),
                ),
            ),
        ));
    }
}
