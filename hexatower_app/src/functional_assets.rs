use std::time::Duration;

use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use core_game_logic::{
    CreationParameters,
    cards::CardId,
    markets::MarketId,
    orders::OrderId,
    players::{ActivePlayer, PlayerData},
};
use thiserror::Error;

use crate::{
    OperatingPlayer,
    inputs_interface::{ActionInputManager, EffectsQueue, NextEffectStartsIn},
    main_menu::BoardSetupInstructions,
    vis_pieces::visual_piece_archetypes_storage::{BasePlatesDirectory, VisualPieceArchetype},
};

#[derive(Debug, ScheduleLabel, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SetUpBoard;

pub type PlayerNames = PlayerData<String>;

pub fn create_board(
    commands: &mut Commands,
    asset_server: &mut AssetServer,
    instructions: BoardSetupInstructions,
) -> Result<(), BevyError> {
    let player_names = PlayerNames::new(instructions.player_names);

    let (logical_world, change_log) = CreationParameters {
        board_size: instructions.board_size.ring_count(),
        player_count: player_names.len() as u8,
        all_cards: core_game_logic::logical_testing_assets::LOGICAL_CARDS_FOR_TESTING.into(),
        all_markets: core_game_logic::logical_testing_assets::LOGICAL_MARKETS_FOR_TESTING.into(),
        starting_cards: core_game_logic::logical_testing_assets::STARTING_CARDS_FOR_TESTING.into(),
        piece_archetypes: core_game_logic::logical_testing_assets::LOGICAL_PIECES_FOR_TESTING
            .into(),
        orders: core_game_logic::logical_testing_assets::LOGICAL_ORDERS_FOR_TESTING.into(),
    }
    .create_logical_world();

    commands.insert_resource(OperatingPlayer(
        if let Some(id) = instructions.you_are_player {
            player_names.make_id(id)?
        } else {
            logical_world.resource::<ActivePlayer>().0
        },
    ));

    commands.insert_resource(LogicalWorld(logical_world));
    {
        let mut baseplate_assets = Vec::new();

        for i in 0..player_names.len() {
            baseplate_assets.push(match i {
                0 => asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("base_plates/green_baseplate.glb"),
                ),
                1 => asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("base_plates/red_baseplate.glb"),
                ),
                2 => asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("base_plates/blue_baseplate.glb"),
                ),
                3 => asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("base_plates/yellow_baseplate.glb"),
                ),
                4 => asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("base_plates/purple_baseplate.glb"),
                ),
                5 => asset_server.load::<WorldAsset>(
                    GltfAssetLabel::Scene(0).from_asset("base_plates/white_baseplate.glb"),
                ),

                _ => asset_server
                    .load::<WorldAsset>(GltfAssetLabel::Scene(0).from_asset("VisualError3d.glb")),
            })
        }

        commands.insert_resource(BasePlatesDirectory::new(
            baseplate_assets.into_boxed_slice(),
        ));
    }

    commands.insert_resource(player_names);

    commands.insert_resource(instructions.board_size);

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

    commands.insert_resource(VisOrderDirectory(
        [
            VisOrder {
                image: asset_server.load("order_icons/Knife.png"),
                name: "The Exemplifier".into(),
                tooltip: "A lovely example which does absolutely nothing.".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/crossed_dagger.png"),
                name: "Stabby stabby".into(),
                tooltip: "Is this thing sharp?".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/single_dagger.png"),
                name: "Corrupter".into(),
                tooltip: "Triplets of darkness".into(),
            },
        ]
        .into(),
    ));
    commands.run_schedule(SetUpBoard);

    commands.insert_resource(EffectsQueue::new(change_log));
    commands.insert_resource(NextEffectStartsIn(Timer::new(
        Duration::from_secs_f32(0.0),
        TimerMode::Once,
    )));

    Ok(())
}

#[derive(Resource, Debug)]
pub struct LogicalWorld(pub World);

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
    pub model: Handle<WorldAsset>,
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
