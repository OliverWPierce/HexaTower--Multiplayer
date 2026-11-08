use std::time::Duration;

use bevy::{
    color::palettes::{css::MAGENTA, tailwind::*},
    ecs::schedule::ScheduleLabel,
    prelude::*,
};
use core_game_logic::{
    CreationParameters,
    cards::{CardId, LogicalCard},
    markets::{LogicalMarket, MarketId},
    orders::{LogicalOrder, OrderId},
    pieces::{LogicalPieceArchetype, Orders},
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
        all_cards: LOGICAL_CARDS.into(),
        all_markets: LOGICAL_MARKETS.into(),
        starting_cards: STARTING_CARDS.into(),
        piece_archetypes: LOGICAL_PIECES.into(),
        orders: LOGICAL_ORDERS.into(),
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
        let mut player_colors = Vec::new();

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
            });
            player_colors.push(
                match i {
                    0 => GREEN_700,
                    1 => RED_700,
                    2 => BLUE_700,
                    3 => YELLOW_700,
                    4 => PURPLE_700,
                    5 => GRAY_700,
                    _ => MAGENTA,
                }
                .into(),
            );
        }

        commands.insert_resource(BasePlatesDirectory::new(
            baseplate_assets.into_boxed_slice(),
        ));
        commands.insert_resource(PlayerData::<Color>::new(player_colors.into_boxed_slice()));
    }

    commands.insert_resource(player_names);

    commands.insert_resource(instructions.board_size);

    commands.insert_resource(ActionInputManager::default());

    commands.insert_resource(VisCardDirectory(Box::new([
        VisualCard {
            image: asset_server.load("card_images/spawn_tower.png"),
            name: "Obelisk Genesis".into(),
            tooltip: "Wait, its really this easy to start a country?".into(),
        },
        VisualCard {
            image: asset_server.load("card_images/spawn_drill.png"),
            name: "Gold Mine Genesis".into(),
            tooltip: "Behold! The mighty pickaxe!".into(),
        },
    ])));

    commands.insert_resource(VisMarketDirectory(Box::new([])));

    commands.insert_resource(
        crate::vis_pieces::visual_piece_archetypes_storage::VisualPieceArchetypeDirectory::new(&[
            VisualPieceArchetype {
                model: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("piece_models/Obelisk.glb")),
                name: "Obelisk".into(),
            },
            VisualPieceArchetype {
                model: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("piece_models/Drill.glb")),
                name: "Gold Mine".into(),
            },
        ]),
    );

    commands.insert_resource(VisOrderDirectory(
        [VisOrder {
            image: asset_server.load("order_icons/crossed_dagger.png"),
            name: "Drill!".into(),
            tooltip: "There's nothing foolish coming out of here".into(),
        }]
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

pub const LOGICAL_CARDS: [LogicalCard; 2] = [
    // Spawn tower
    LogicalCard {
        functionality: core_game_logic::cards::CardFunction::SpawnPiece {
            selection_bounds: 1..1,
            piece_archetype: core_game_logic::pieces::ArchetypeId(0),
            restrictions:
                core_game_logic::tile_based_actions::spawn_pieces::SpawningRestrictions::TowerSpawns,
        },
    },
    // Spawn drill
    LogicalCard {
        functionality: core_game_logic::cards::CardFunction::SpawnPiece {
            selection_bounds: 1..1,
            piece_archetype: core_game_logic::pieces::ArchetypeId(1),
            restrictions:
                core_game_logic::tile_based_actions::spawn_pieces::SpawningRestrictions::StandardRestrictions,
        },
    },
];

pub const LOGICAL_MARKETS: [LogicalMarket; 0] = [];

pub const STARTING_CARDS: [CardId; 2] = [CardId(0), CardId(1)];

pub const LOGICAL_PIECES: [LogicalPieceArchetype; 2] = [
    //tower
    LogicalPieceArchetype {
        max_health: 250,
        orders_per_round: 0,
        orders: Orders([None; 5]),
        gives_extra_player_order: true,
        default_monetary_value: 10,
        is_win_condition: true,
        is_spawnpoint: true,
    },
    //drill
    LogicalPieceArchetype {
        max_health: 25,
        orders_per_round: 1,
        orders: Orders([Some(OrderId(0)), None, None, None, None]),
        gives_extra_player_order: false,
        default_monetary_value: 2,
        is_win_condition: false,
        is_spawnpoint: false,
    },
];

pub const LOGICAL_ORDERS: [LogicalOrder; 1] = [LogicalOrder {
    functionality: core_game_logic::orders::OrderFunction::GiveOwnedCurrency { ammount: 3 },
}];
