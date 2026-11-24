use std::time::Duration;

use bevy::{
    color::palettes::{css::MAGENTA, tailwind::*},
    ecs::schedule::ScheduleLabel,
    prelude::*,
};
use core_game_logic::{
    CreationParameters,
    cards::{CardId, LogicalCard},
    markets::{CardPrice, LogicalMarket, MarketId},
    orders::{LogicalOrder, OrderFunction, OrderId},
    pieces::{LogicalPieceArchetype, Orders},
    players::{ActivePlayer, PlayerData},
    tile_based_actions::move_piece::MovementMethod,
};
use thiserror::Error;

use crate::{
    OperatingPlayer,
    inputs_interface::{ActionInputManager, EffectsQueue, NextEffectStartsIn},
    main_menu::BoardSetupInstructions,
    vis_pieces::visual_piece_archetypes_storage::{BasePlatesDirectory, VisualPieceArchetype},
};

pub const MAX_VISUAL_PLAYERS: u8 = 6;

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
        all_markets: LOGICAL_MARKETS
            .iter()
            .copied()
            .zip(instructions.initial_market_tiles)
            .collect::<Box<_>>(),
        starting_cards: STARTING_CARDS.into(),
        piece_archetypes: LOGICAL_PIECES.into(),
        orders: LOGICAL_ORDERS.into(),
        initial_special_tiles: instructions.initial_special_tiles,
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
        VisualCard {
            image: asset_server.load("card_images/spawn_spaceship.png"),
            name: "Photon Cruiser Genesis".into(),
            tooltip: "Are we ther... oh we're here.".into(),
        },
        VisualCard {
            image: asset_server.load("card_images/spawn_cannon_car.png"),
            name: "Cannon Car Genesis".into(),
            tooltip: "Cannon + car = vroom + boom. Awesome.".into(),
        },
        VisualCard {
            image: asset_server.load("card_images/market_1.png"),
            name: "Agora Emporion".into(),
            tooltip: "The foundational industry of society. Occupy it!".into(),
        },
        VisualCard {
            image: asset_server.load("card_images/spawn_rover.png"),
            name: "Rolling Saucer Genesis".into(),
            tooltip: "Alien tech, grounded, and manned by knights. Logically...".into(),
        },
        VisualCard {
            image: asset_server.load("card_images/spawn_station.png"),
            name: "Command Point Alpha Genesis".into(),
            tooltip: "A forward base, complete with a hospital".into(),
        },
        VisualCard {
            image: asset_server.load("card_images/spawn_wizard.png"),
            name: "Mage Genesis".into(),
            tooltip: "He seems... otherworldly. Also, he likes shrimp tacos.".into(),
        },
        VisualCard {
            image: asset_server.load("card_images/health_upgrade.png"),
            name: "Weights".into(),
            tooltip: "You do actually have to use them.".into(),
        },
        VisualCard {
            image: asset_server.load("card_images/orders_upgrade.png"),
            name: "Bigger Mailbox".into(),
            tooltip: "Do I actually have to read all of these?".into(),
        },
        VisualCard {
            image: asset_server.load("card_images/gold_tile.png"),
            name: "Troves Constructor".into(),
            tooltip: "Money doesn't grow on trees, it grows underground.".into(),
        },
    ])));

    commands.insert_resource(VisMarketDirectory(Box::new([
        VisMarket {
            model: asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("market_models/purple_yellow.glb")),
            name: "The Agora".into(),
            description: "All the essentials for civilizations at war".into(),
        },
        VisMarket {
            model: asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("market_models/maroon_teal.glb")),
            name: "The Armory".into(),
            description: "Our values: Integrity. Industry. Backstabbing".into(),
        },
        VisMarket {
            model: asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("market_models/green_purple.glb")),
            name: "Hermit's Haunt".into(),
            description: "Most struggle to describe the wares shold here.".into(),
        },
    ])));

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
            VisualPieceArchetype {
                model: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("piece_models/Spaceship.glb")),
                name: "Photon Cruiser".into(),
            },
            VisualPieceArchetype {
                model: asset_server
                    .load(GltfAssetLabel::Scene(0).from_asset("piece_models/Cannon Car.glb")),
                name: "Cannon Car".into(),
            },
            VisualPieceArchetype {
                model: asset_server.load(
                    GltfAssetLabel::Scene(0).from_asset("piece_models/Magnetic Hockey Rover.glb"),
                ),
                name: "Rolling Saucer".into(),
            },
            VisualPieceArchetype {
                model: asset_server.load(
                    GltfAssetLabel::Scene(0)
                        .from_asset("piece_models/Cosmic Cargo Space Station.glb"),
                ),
                name: "Command Point Alpha".into(),
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
                image: asset_server.load("order_icons/Pickaxe.png"),
                name: "Drill!".into(),
                tooltip: "The glimmering abyss calls to us".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/rotate.png"),
                name: "Rotate!".into(),
                tooltip: "We all have a joyful little ballerina inside -- Too Tzu".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/adjacent_move.png"),
                name: "March!".into(),
                tooltip: "One step at a time".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/dash.png"),
                name: "Dash!".into(),
                tooltip: "Please remain seated while spacehip is in motion".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/sword.png"),
                name: "Decimator!".into(),
                tooltip: "Caution: serious injury or death will occur if used correctly".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/flint_arrow.png"),
                name: "Cannon!".into(),
                tooltip: "Cannon + car = vroom + boom. Awesome.".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/rover_move.png"),
                name: "Advance!".into(),
                tooltip: "Hustle not included.".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/battering_ram.png"),
                name: "Battering Ram!".into(),
                tooltip: "It is advised to stand clear".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/single_heart.png"),
                name: "Repairman!".into(),
                tooltip: "What a mess! I'll fix this right up, eventually. ".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/two_hearts.png"),
                name: "Crash Cart!".into(),
                tooltip: "Quickly now... and don't run anyone over.".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/yellow_wand.png"),
                name: "Droblus' Draft!".into(),
                tooltip: "No one knows what its made of, but it seems to help.".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/magic_dagger.png"),
                name: "Sheen of Dusk!".into(),
                tooltip: "Some question if a shadow is an effective weapon choice.".into(),
            },
            VisOrder {
                image: asset_server.load("order_icons/magic_staff.png"),
                name: "Poof!".into(),
                tooltip: "Didn't see that coming, did you?".into(),
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

pub const LOGICAL_CARDS: [LogicalCard; 11] = [
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
                core_game_logic::tile_based_actions::spawn_pieces::SpawningRestrictions::StandardRestrictions {depth: 1},
        },
    },
    // Spawn spaceship
    LogicalCard {
        functionality: core_game_logic::cards::CardFunction::SpawnPiece {
            selection_bounds: 1..1,
            piece_archetype: core_game_logic::pieces::ArchetypeId(2),
            restrictions:
                core_game_logic::tile_based_actions::spawn_pieces::SpawningRestrictions::StandardRestrictions {depth: 1},
        },
    },
    LogicalCard {
        functionality: core_game_logic::cards::CardFunction::SpawnPiece {
            selection_bounds: 1..1,
            piece_archetype: core_game_logic::pieces::ArchetypeId(3),
            restrictions:
                core_game_logic::tile_based_actions::spawn_pieces::SpawningRestrictions::StandardRestrictions {depth: 1},
        },
    },
    LogicalCard {
        functionality: core_game_logic::cards::CardFunction::SpawnMarket { selection_bounds: 1..1, market: MarketId(0) },
    },
    LogicalCard {
        functionality: core_game_logic::cards::CardFunction::SpawnPiece {
            selection_bounds: 1..1,
            piece_archetype: core_game_logic::pieces::ArchetypeId(4),
            restrictions:
                core_game_logic::tile_based_actions::spawn_pieces::SpawningRestrictions::StandardRestrictions {depth: 1},
        },
    },
    LogicalCard {
        functionality: core_game_logic::cards::CardFunction::SpawnPiece {
            selection_bounds: 1..1,
            piece_archetype: core_game_logic::pieces::ArchetypeId(5),
            restrictions:
                core_game_logic::tile_based_actions::spawn_pieces::SpawningRestrictions::StandardRestrictions {depth: 2},
        },
    },
    LogicalCard {
        functionality: core_game_logic::cards::CardFunction::SpawnPiece {
            selection_bounds: 1..1,
            piece_archetype: core_game_logic::pieces::ArchetypeId(6),
            restrictions:
                core_game_logic::tile_based_actions::spawn_pieces::SpawningRestrictions::StandardRestrictions {depth: 1},
        },
    },
    LogicalCard { functionality: core_game_logic::cards::CardFunction::UpgradeHealth { selection_bounds: 1..2 } },
    LogicalCard { functionality: core_game_logic::cards::CardFunction::UpgradePieceOrdersPerRound { selection_bounds: 1..1 } },
    LogicalCard { functionality: core_game_logic::cards::CardFunction::TileConversionToSingleType { selection_bounds: 1..1, target_type: core_game_logic::tiles::TileType::Gold }},
];

pub const LOGICAL_MARKETS: [LogicalMarket; 3] = [
    LogicalMarket([
        (CardId(1), CardPrice(7)),
        (CardId(2), CardPrice(5)),
        (CardId(3), CardPrice(4)),
    ]),
    LogicalMarket([
        (CardId(6), CardPrice(10)),
        (CardId(9), CardPrice(6)),
        (CardId(10), CardPrice(5)),
    ]),
    LogicalMarket([
        (CardId(7), CardPrice(7)),
        (CardId(8), CardPrice(6)),
        (CardId(5), CardPrice(4)),
    ]),
];

pub const STARTING_CARDS: [CardId; 1] = [CardId(0)];

pub const LOGICAL_PIECES: [LogicalPieceArchetype; 7] = [
    //tower
    LogicalPieceArchetype {
        max_health: 250,
        orders_per_round: 1,
        orders: Orders([Some(OrderId(8)), None, None, None, None]),
        gives_extra_player_order: true,
        default_monetary_value: 10,
        is_win_condition: true,
        is_spawnpoint: true,
        creates_market: Some(MarketId(0)),
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
        creates_market: None,
    },
    //spaceship
    LogicalPieceArchetype {
        max_health: 30,
        orders_per_round: 1,
        orders: Orders([
            Some(OrderId(1)),
            Some(OrderId(2)),
            Some(OrderId(3)),
            Some(OrderId(4)),
            None,
        ]),
        gives_extra_player_order: false,
        default_monetary_value: 4,
        is_win_condition: false,
        is_spawnpoint: false,
        creates_market: None,
    },
    //cannon car
    LogicalPieceArchetype {
        max_health: 35,
        orders_per_round: 1,
        orders: Orders([
            Some(OrderId(1)),
            Some(OrderId(2)),
            Some(OrderId(5)),
            None,
            None,
        ]),
        gives_extra_player_order: false,
        default_monetary_value: 3,
        is_win_condition: false,
        is_spawnpoint: false,
        creates_market: None,
    },
    //Rover
    LogicalPieceArchetype {
        max_health: 50,
        orders_per_round: 1,
        orders: Orders([
            Some(OrderId(1)),
            Some(OrderId(6)),
            Some(OrderId(7)),
            None,
            None,
        ]),
        gives_extra_player_order: false,
        default_monetary_value: 3,
        is_win_condition: false,
        is_spawnpoint: false,
        creates_market: None,
    },
    //Space Station
    LogicalPieceArchetype {
        max_health: 20,
        orders_per_round: 1,
        orders: Orders([Some(OrderId(9)), None, None, None, None]),
        gives_extra_player_order: true,
        default_monetary_value: 6,
        is_win_condition: false,
        is_spawnpoint: true,
        creates_market: None,
    },
    //wizard
    LogicalPieceArchetype {
        max_health: 30,
        orders_per_round: 1,
        orders: Orders([
            Some(OrderId(1)),
            Some(OrderId(2)),
            Some(OrderId(10)),
            Some(OrderId(11)),
            Some(OrderId(12)),
        ]),
        gives_extra_player_order: false,
        default_monetary_value: 6,
        is_win_condition: false,
        is_spawnpoint: false,
        creates_market: None,
    },
];

pub const LOGICAL_ORDERS: [LogicalOrder; 13] = [
    LogicalOrder {
        functionality: OrderFunction::GiveOwnedCurrency { ammount: 3 },
    },
    LogicalOrder {
        functionality: OrderFunction::Rotate,
    },
    LogicalOrder {
        functionality: OrderFunction::BasicMovement(MovementMethod::Adjacent { depth: 1 }),
    },
    LogicalOrder {
        functionality: OrderFunction::BasicMovement(MovementMethod::Forward { depth: 2 }),
    },
    LogicalOrder {
        functionality: OrderFunction::DamageAdjacent {
            depth: 1,
            selection_range: 1..1,
            damage: 30,
        },
    },
    LogicalOrder {
        functionality: OrderFunction::DamageForward {
            depth: 2,
            selection_range: 1..1,
            damage: 20,
        },
    },
    LogicalOrder {
        functionality: OrderFunction::BasicMovement(MovementMethod::Forward { depth: 1 }),
    },
    LogicalOrder {
        functionality: OrderFunction::DamageForward {
            depth: 2,
            selection_range: 1..1,
            damage: 60,
        },
    },
    LogicalOrder {
        functionality: OrderFunction::HealAdjacentAndSelf {
            amount: 5,
            allow_self: true,
            selection_range: 1..1,
            selection_depth: 0,
        },
    },
    LogicalOrder {
        functionality: OrderFunction::HealAdjacentAndSelf {
            amount: 5,
            allow_self: true,
            selection_range: 1..1,
            selection_depth: 1,
        },
    },
    LogicalOrder {
        functionality: OrderFunction::HealAdjacentAndSelf {
            amount: 10,
            allow_self: false,
            selection_range: 1..1,
            selection_depth: 1,
        },
    },
    LogicalOrder {
        functionality: OrderFunction::Attack3Fan {
            damage: 20,
            selection_range: 1..1,
        },
    },
    LogicalOrder {
        functionality: OrderFunction::SwapSelf,
    },
];
