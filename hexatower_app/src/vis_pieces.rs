use bevy::prelude::*;
use core_game_logic::{
    requests::ActionEffect,
    tile_mapping::{HexVector2d, TileId},
};

use crate::{
    inputs_interface::EffectToDisplay,
    vis_pieces::visual_piece_archetypes_storage::{
        BasePlatesDirectory, VisualPieceArchetypeDirectory,
    },
};

pub struct VisPiecesPlugin;

impl Plugin for VisPiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_visuals,
                update_backend_rotation_changes,
                start_piece_move,
            )
                .run_if(resource_exists_and_changed::<EffectToDisplay>),
        );
    }
}

pub mod visual_piece_archetypes_storage {

    use bevy::prelude::*;
    use core_game_logic::{pieces::ArchetypeId, players::PlayerData};
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

    pub type BasePlatesDirectory = PlayerData<Handle<WorldAsset>>;
}

#[derive(Debug, Component)]
pub struct VisOccupies(pub TileId);

const BASEPLATE_HEIGHT: f32 = 0.115;

fn spawn_visuals(
    effect: Res<EffectToDisplay>,
    mut commands: Commands,
    base_plates: Res<BasePlatesDirectory>,
    piece_details: Res<VisualPieceArchetypeDirectory>,
) -> Result<(), BevyError> {
    let ActionEffect::SpawnedPiece {
        tile,
        owner: player,
        archetype,
        facing_direction,
    } = effect.0
    else {
        return Ok(());
    };
    let horizontal_location: Vec2 = core_game_logic::tile_mapping::HexVector2d::from(tile).into();
    commands.spawn((
        VisOccupies(tile),
        Transform::from_translation(Vec3 {
            x: horizontal_location.x,
            y: 0.0,
            z: horizontal_location.y,
        })
        .looking_to(Vec3::from(HexVector2d::from(facing_direction)), Vec3::Y),
        WorldAssetRoot(base_plates.get(player).clone()),
        children![
            WorldAssetRoot(piece_details.get_visual_details(archetype)?.model.clone()),
            Transform::from_translation(Vec3 {
                x: 0.0,
                y: BASEPLATE_HEIGHT,
                z: 0.0
            })
        ],
    ));

    Ok(())
}

fn update_backend_rotation_changes(
    effect: Res<EffectToDisplay>,
    mut pieces: Query<(&mut Transform, &VisOccupies)>,
) -> Result<(), BevyError> {
    let ActionEffect::PieceRotated {
        on_tile,
        new_rotation,
    } = effect.0
    else {
        return Ok(());
    };

    let (mut transform, _) = pieces
        .iter_mut()
        .find(|(_, tile)| tile.0 == on_tile)
        .ok_or("No visual piece occupies this tile")?;

    transform.look_to(Vec3::from(HexVector2d::from(new_rotation)), Vec3::Y);

    Ok(())
}

fn start_piece_move(
    effect: Res<EffectToDisplay>,
    mut pieces: Query<(&mut Transform, &mut VisOccupies)>,
) -> Result<(), BevyError> {
    let ActionEffect::PieceMoved { from_tile, to_tile } = effect.0 else {
        return Ok(());
    };

    let (mut transform, mut piece_occupies) = pieces
        .iter_mut()
        .find(|(.., tile)| tile.0 == from_tile)
        .ok_or("No visual piece occupies the tile a logical piece has moved from.")?;

    piece_occupies.0 = to_tile;
    transform.translation = HexVector2d::from(to_tile).into();

    Ok(())
}
