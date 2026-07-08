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

        app.add_message::<RotatePieceMessage>();

        app.add_systems(
            Update,
            (spawn_visuals, update_backend_rotation_changes).chain(),
        );
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
        pub model: Handle<WorldAsset>,
        pub name: String,
    }

    #[derive(Debug, Resource)]
    pub struct BasePlatesDirectory(Box<[Handle<WorldAsset>]>);

    #[derive(Debug, Error)]
    #[error("Could not find a baseplate model for player {0:?}.")]
    pub struct MissingBaseplate(PlayerId);

    impl BasePlatesDirectory {
        pub fn get_base_plate(
            &self,
            id: PlayerId,
        ) -> Result<&Handle<WorldAsset>, MissingBaseplate> {
            self.0.get(id.0 as usize).ok_or(MissingBaseplate(id))
        }

        pub fn new(models: &[Handle<WorldAsset>]) -> Self {
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
pub struct VisOccupies(pub TileId);

const BASEPLATE_HEIGHT: f32 = 0.115;

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
            VisOccupies(spawn.tile),
            Transform::from_translation(Vec3 {
                x: horizontal_location.x,
                y: 0.0,
                z: horizontal_location.y,
            })
            .looking_to(Vec3::from(HexVector2d::from(spawn.direction)), Vec3::Y),
            WorldAssetRoot(base_plates.get_base_plate(spawn.owner)?.clone()),
            children![
                WorldAssetRoot(
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
pub struct RotatePieceMessage {
    pub on_tile: TileId,
    pub in_direction: FacingHexDirection,
}

fn update_backend_rotation_changes(
    mut pieces_to_rotate: MessageReader<RotatePieceMessage>,
    mut pieces: Query<(&mut Transform, &VisOccupies)>,
) {
    for &RotatePieceMessage {
        on_tile,
        in_direction,
    } in pieces_to_rotate.read()
    {
        if let Some((mut transform, _)) = pieces.iter_mut().find(|(_, tile)| tile.0 == on_tile) {
            transform.look_to(Vec3::from(HexVector2d::from(in_direction)), Vec3::Y);
        } else {
            warn!(
                "Tried to rotate the visual piece, but no visual piece was found on tile {on_tile:?}",
            )
        };
        continue;
    }
}
