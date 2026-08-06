use bevy::{math::VectorSpace, prelude::*};
use core_game_logic::{
    requests::ActionEffect,
    tile_mapping::{HexVector2d, TileId},
};

use crate::{
    inputs_interface::EffectToDisplay,
    vis_effect_reactions::{
        AnimatedProperty, AnimatedPropertyInterval, AnimatedScale, AnimatedTranslation,
    },
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
                despawn_piece,
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

    const SCALE_IN_DUR: f32 = 1.0;
    commands.spawn((
        VisOccupies(tile),
        Transform::from_translation(Vec3 {
            x: horizontal_location.x,
            y: 0.0,
            z: horizontal_location.y,
        })
        .looking_to(Vec3::from(HexVector2d::from(facing_direction)), Vec3::Y)
        .with_scale(Vec3::ZERO),
        WorldAssetRoot(base_plates.get(player).clone()),
        AnimatedScale(
            AnimatedProperty::new_seamless(
                Vec3::ZERO,
                [AnimatedPropertyInterval {
                    next_value: Vec3::ONE,
                    duration: SCALE_IN_DUR,
                    mode: EaseFunction::BackOut,
                }]
                .into(),
                0.0,
            ),
            false,
        ),
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
    mut pieces: Query<(Entity, &mut VisOccupies)>,
    mut commands: Commands,
) -> Result<(), BevyError> {
    let ActionEffect::PieceMoved { from_tile, to_tile } = effect.0 else {
        return Ok(());
    };

    const MOVE_SPEED: f32 = 0.3;

    let (ent, mut piece_occupies) = pieces
        .iter_mut()
        .find(|(.., tile)| tile.0 == from_tile)
        .ok_or("No visual piece occupies the tile a logical piece has moved from.")?;

    commands
        .entity(ent)
        .insert(AnimatedTranslation(AnimatedProperty::new_seamless(
            Vec3::from(HexVector2d::from(from_tile)),
            [AnimatedPropertyInterval {
                next_value: Vec3::from(HexVector2d::from(to_tile)),
                duration: MOVE_SPEED
                    * Vec3::from(HexVector2d::from(from_tile))
                        .distance(Vec3::from(HexVector2d::from(to_tile))),
                mode: EaseFunction::SmoothStep,
            }]
            .into(),
            0.0,
        )));
    piece_occupies.0 = to_tile;

    Ok(())
}

fn despawn_piece(
    mut commands: Commands,
    effect: Res<EffectToDisplay>,
    pieces: Query<(Entity, &VisOccupies)>,
) {
    let ActionEffect::PieceKilled { on_tile } = effect.0 else {
        return;
    };
    let Some((piece, ..)) = pieces
        .iter()
        .find(|(_, VisOccupies(piece_is_on_tile))| *piece_is_on_tile == on_tile)
    else {
        return;
    };

    const SCALE_OUT_ANIM_DUR: f32 = 1.0;

    commands.entity(piece).insert(AnimatedScale(
        AnimatedProperty::new_seamless(
            Vec3::ONE,
            [AnimatedPropertyInterval {
                next_value: Vec3::ZERO,
                duration: SCALE_OUT_ANIM_DUR,
                mode: EaseFunction::BackIn,
            }]
            .into(),
            0.0,
        ),
        true,
    ));
}
