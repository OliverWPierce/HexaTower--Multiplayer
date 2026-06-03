use bevy::prelude::*;

pub struct VisPiecesPlugin;

impl Plugin for VisPiecesPlugin {
    fn build(&self, app: &mut App) {
        ()
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
