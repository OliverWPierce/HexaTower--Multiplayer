use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use core_game_logic::{CreationParameters, cards::CardId, players::PlayerId};

use crate::{
    DisplayPlayer, OperatingPlayer,
    vis_pieces::visual_piece_archetypes_storage::VisualPieceArchetype, vis_tiles::BoardSize,
};

pub struct StartupPlugin;

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, tmp_startup);
    }
}
#[derive(Debug, ScheduleLabel, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SetUpBoard;

fn tmp_startup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.insert_resource(GameCreationSettings {
        board_size: BoardSize::Standard,
    });
    commands.insert_resource(DisplayPlayer(PlayerId(0)));
    commands.insert_resource(OperatingPlayer(Some(PlayerId(0))));

    commands.insert_resource(VisCardDirectory(Box::new([
        VisualCard {
            image: asset_server.load("item_images/blue_potion.png"),
            name: "ZERO".into(),
            tooltip: "Tooltip for card 0.".into(),
        },
        VisualCard {
            image: asset_server.load("item_images/red_glow_potion.png"),
            name: "ONE".into(),
            tooltip: "Tooltip for card 1.".into(),
        },
        VisualCard {
            image: asset_server.load("item_images/green_potion.png"),
            name: "TWO".into(),
            tooltip: "Tooltip for card 2.".into(),
        },
        VisualCard {
            image: asset_server.load("item_images/purple_potion.png"),
            name: "THREE".into(),
            tooltip: "Tooltip for card 3.".into(),
        },
        VisualCard {
            image: asset_server.load("item_images/blue_potion.png"),
            name: "FOUR".into(),
            tooltip: "Tooltip for card 4.".into(),
        },
        VisualCard {
            image: asset_server.load("item_images/green_potion.png"),
            name: "FIVE".into(),
            tooltip: "Tooltip for card 5.".into(),
        },
    ])));

    commands.insert_resource(
        crate::vis_pieces::visual_piece_archetypes_storage::VisualPieceArchetypeDirectory::new(&[
            VisualPieceArchetype {
                model: todo!(),
                name: "Archetype TWO".into(),
            },
            VisualPieceArchetype {
                model: todo!(),
                name: "Archetype ONE".into(),
            },
        ]),
    );

    commands.insert_resource(
        crate::vis_pieces::visual_piece_archetypes_storage::BasePlatesDirectory::new(&[
            asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("base_plates/green_baseplate.glb")),
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("base_plates/red_baseplate.glb")),
            asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("base_plates/yellow_baseplate.glb")),
        ]),
    );

    commands.insert_resource(LogicalWorld(CreationParameters::testing_default()));

    commands.run_schedule(SetUpBoard);
}

#[derive(Resource, Debug)]
pub struct LogicalWorld(pub World);

#[derive(Debug, Resource)]
pub struct GameCreationSettings {
    pub board_size: BoardSize,
}
#[derive(Debug)]
pub struct VisualCard {
    pub image: Handle<Image>,
    pub name: String,
    pub tooltip: String,
}

#[derive(Debug, Resource)]
pub struct VisCardDirectory(pub Box<[VisualCard]>);
#[derive(Debug, Component)]
pub struct VisualCardId(pub CardId);
