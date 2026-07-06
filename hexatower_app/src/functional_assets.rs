use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use core_game_logic::{
    CreationParameters, cards::CardId, markets::MarketId, orders::OrderId, players::PlayerId,
};
use thiserror::Error;

use crate::{
    OperatingPlayer,
    inputs_interface::{ActionInputManager, MultiplayerNetworkingMode},
    vis_pieces::visual_piece_archetypes_storage::VisualPieceArchetype,
    vis_tiles::BoardSize,
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
    commands.insert_resource(OperatingPlayer(PlayerId(0)));
    commands.insert_resource(MultiplayerNetworkingMode::SingleDevice);

    commands.insert_resource(ActionInputManager::default());

    commands.insert_resource(VisCardDirectory(Box::new([
        VisualCard {
            image: asset_server.load("item_images/blue_potion.png"),
            name: "Elixer of Youth".into(),
            tooltip: "Command Z! command Zeee!!".into(),
        },
        VisualCard {
            image: asset_server.load("item_images/red_glow_potion.png"),
            name: "Draft of Orthendale".into(),
            tooltip: "Realestate is my specialty!".into(),
        },
        VisualCard {
            image: asset_server.load("item_images/green_potion.png"),
            name: "Magic Sauce".into(),
            tooltip: "Enjoy your very own piece".into(),
        },
        VisualCard {
            image: asset_server.load("order_icons/single_dagger.png"),
            name: "Market Spawner".into(),
            tooltip: "All your needed wares sold here!".into(),
        },
        VisualCard {
            image: asset_server.load("item_images/blue_potion.png"),
            name: "Radioactive Slurry".into(),
            tooltip: "Mind ye placement".into(),
        },
        VisualCard {
            image: asset_server.load("item_images/green_potion.png"),
            name: "Potion of Punctuation".into(),
            tooltip: "Something witty about this piece will be established later.".into(),
        },
    ])));

    commands.insert_resource(VisMarketDirectory(Box::new([
        VisMarket {
            model: asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("market_models/gray_yellow_market.glb")),
            name: "Tarmart".into(),
            description: "Basic supplies sold here!".into(),
        },
        VisMarket {
            model: asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("market_models/purple_market.glb")),
            name: "The Stalls of Angora".into(),
            description: "Only the finest wares.".into(),
        },
    ])));

    commands.insert_resource(
        crate::vis_pieces::visual_piece_archetypes_storage::VisualPieceArchetypeDirectory::new(&[
            VisualPieceArchetype {
                model: asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("piece_models/Magnetic Hockey Rover.glb"),
                ),
                name: "Jupiter Rover".into(),
            },
            VisualPieceArchetype {
                model: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("piece_models/Obelisk.glb")),
                name: "Obelisk".into(),
            },
            VisualPieceArchetype {
                model: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("piece_models/Wizard Hat.glb")),
                name: "Mage of Terolark".into(),
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

    commands.insert_resource(VisOrderDirectory(
        [
            VisOrder {
                image: asset_server.load("order_icons/Knife.png"),
                name: "The Exemplifier".into(),
                tooltip: "A lovely example which does absolutely nothing.".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/crossed_dagger.png"),
                name: "Remedy".into(),
                tooltip: "Good as new".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/single_dagger.png"),
                name: "Corrupter".into(),
                tooltip: "Triplets of darkness".into(),
            },
        ]
        .into(),
    ));

    commands.insert_resource(LogicalWorld(CreationParameters::testing_default()));

    commands.run_schedule(SetUpBoard);
}

#[derive(Resource, Debug)]
pub struct LogicalWorld(pub World);

#[derive(Debug, Resource)]
pub struct GameCreationSettings {
    pub board_size: BoardSize,
}
#[derive(Debug, Clone)]
pub struct VisualCard {
    pub image: Handle<Image>,
    pub name: String,
    pub tooltip: String,
}

#[derive(Debug, Resource)]
pub struct VisCardDirectory(Box<[VisualCard]>);

#[derive(Debug, Error)]
#[error{"Tried to retrieve card data using an invalid CardId {0:?}."}]
pub struct VisInvaildCardIdErr(pub CardId);

impl VisCardDirectory {
    pub fn get_card(&self, id: CardId) -> Result<&VisualCard, VisInvaildCardIdErr> {
        self.0.get(id.0 as usize).ok_or(VisInvaildCardIdErr(id))
    }
}

#[derive(Debug, Component)]
pub struct VisualCardId(pub CardId);

#[derive(Debug)]
pub struct VisOrder {
    pub image: Handle<Image>,
    pub name: String,
    pub tooltip: String,
}

#[derive(Debug, Resource)]
pub struct VisOrderDirectory(Box<[VisOrder]>);

#[derive(Debug, Error)]
#[error{"Tried to retrieve visual order data using an invalid OrderId {0:?}."}]
pub struct VisInvaildOrderIdErr(pub OrderId);

impl VisOrderDirectory {
    pub fn get_order(&self, id: OrderId) -> Result<&VisOrder, VisInvaildOrderIdErr> {
        self.0.get(id.0 as usize).ok_or(VisInvaildOrderIdErr(id))
    }
}

#[derive(Debug)]
pub struct VisMarket {
    pub model: Handle<Scene>,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Resource)]
pub struct VisMarketDirectory(Box<[VisMarket]>);

#[derive(Debug, Error)]
#[error{"Tried to retrieve visual order data using an invalid OrderId {0:?}."}]
pub struct VisInvaildMarketIdErr(pub MarketId);

impl VisMarketDirectory {
    pub fn get_market(&self, id: MarketId) -> Result<&VisMarket, VisInvaildMarketIdErr> {
        self.0.get(id.0 as usize).ok_or(VisInvaildMarketIdErr(id))
    }
}
