use bevy::{asset::AssetLoader, ecs::schedule::ScheduleLabel, prelude::*};
use core_game_logic::{
    cards::{CardFunction, CardId, LogicalCard},
    markets::{CardPrice, LogicalMarket},
    tiles::TileType,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{GameSetupInstructions, PreSetUpBoard};

/// This plugin handles loading and sorting all assets that have a functional impact on gameplay.
pub struct LogicalAssetLoadingPlugin;

impl Plugin for LogicalAssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            syst_watch_for_pack_load.run_if(resource_exists::<WaitingOnAssetsToLoad>),
        );

        app.add_systems(PreSetUpBoard, syst_load_packs);

        app.init_schedule(AssetsFinishedLoading);
    }
}

#[derive(Debug, Resource)]
struct ActivePackAssets(Box<[Handle<PackAsset>]>);

#[derive(Debug, Asset, TypePath)]
struct PackAsset {
    starting_cards: Box<[Handle<IntermediateCard>]>,
    starting_markets: Box<[Handle<IntermediateMarket>]>,
}

#[derive(Debug, Asset, TypePath)]
struct IntermediateCard {
    name: String,
    functionality: IntermediateCardFunction,
}
#[derive(Debug)]
enum IntermediateCardFunction {
    ConvertTileTo { target_tile: TileType },
    SpawnMarket(Handle<IntermediateMarket>),
}

#[derive(Debug, Asset, TypePath)]
struct IntermediateMarket {
    name: String,
    offers: [(Handle<IntermediateCard>, CardPrice); 3],
}

#[derive(Debug)]
pub struct PackPathsToLoad(Vec<String>);

fn syst_load_packs(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<GameSetupInstructions>,
) {
    commands.insert_resource(ActivePackAssets(
        settings
            .packs
            .0
            .iter()
            .map(|path| asset_server.load::<PackAsset>(path))
            .collect::<Vec<Handle<PackAsset>>>()
            .into_boxed_slice(),
    ));
    commands.insert_resource(WaitingOnAssetsToLoad);
}
#[derive(Debug, Resource)]
struct WaitingOnAssetsToLoad;

fn syst_watch_for_pack_load(
    asset_server: Res<AssetServer>,
    packs_to_check: Res<ActivePackAssets>,
    mut commands: Commands,
) {
    let is_done = packs_to_check
        .0
        .iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle));

    if is_done {
        commands.remove_resource::<WaitingOnAssetsToLoad>();
        commands.run_schedule(AssetsFinishedLoading);
    }
}

#[derive(Debug, ScheduleLabel, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AssetsFinishedLoading;

#[derive(Debug)]
struct VisualCard {
    name: String,
}
#[derive(Debug, Resource)]
struct VisCards(Box<[VisualCard]>);

#[derive(Debug, Resource)]
struct VisMarkets(Box<[VisualMarket]>);

#[derive(Debug)]
struct VisualMarket {
    name: String,
}

struct BackendAssetData {
    cards: Box<[LogicalCard]>,
    initial_player_inventory: Box<[CardId]>,
    markets: Box<[LogicalMarket]>,
}

fn sort_cards(
    all_cards: Res<Assets<IntermediateCard>>,
    all_markets: Res<Assets<IntermediateMarket>>,
    packs: Res<Assets<PackAsset>>,
    mut commands: Commands,
) -> BackendAssetData {
    let sorted_cards = {
        let mut pre_sorted = all_cards.iter().collect::<Vec<_>>();
        pre_sorted.sort_by(|(_, card1), (_, card2)| card1.name.cmp(&card2.name));
        pre_sorted
    };

    commands.insert_resource(VisCards(
        sorted_cards
            .iter()
            .map(|(_, card_data)| VisualCard {
                name: card_data.name.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    ));

    let sorted_markets = {
        let mut pre_sorted = all_markets.iter().collect::<Vec<_>>();
        pre_sorted.sort_by(|(_, card1), (_, card2)| card1.name.cmp(&card2.name));
        pre_sorted
    };

    commands.insert_resource(VisMarkets(
        sorted_markets
            .iter()
            .map(|(_, market_data)| VisualMarket {
                name: market_data.name.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    ));

    let mut logical_cards = Vec::new();

    for (_, card) in sorted_cards.as_slice() {
        let backend_function = match &card.functionality {
            IntermediateCardFunction::ConvertTileTo { target_tile } => {
                CardFunction::TileConversionToSingleType {
                    selection_bounds: 1..2,
                    target_type: target_tile.clone(),
                }
            }
            IntermediateCardFunction::SpawnMarket(handle) => {
                let market_asset_id = handle.id();
                let market_id = core_game_logic::markets::MarketId(
                    sorted_markets
                        .iter()
                        .enumerate()
                        .find(|(_, (id, _))| *id == market_asset_id)
                        .unwrap()
                        .0 as u32,
                );

                CardFunction::SpawnMarket {
                    selection_bounds: 1..2,
                    market: market_id,
                }
            }
        };

        logical_cards.push(LogicalCard {
            functionality: backend_function,
        })
    }

    let mut logical_markets = Vec::new();

    for (_, market) in sorted_markets {
        let x = core_game_logic::markets::LogicalMarket(market.offers.clone().map(
            |(card_handle, price)| {
                let card_asset_id = card_handle.id();
                let id_of_card_offered = CardId(
                    sorted_cards
                        .iter()
                        .enumerate()
                        .find(|(_, (asset_id, _))| *asset_id == card_asset_id)
                        .unwrap()
                        .0 as u32,
                );

                (id_of_card_offered, price)
            },
        ));

        logical_markets.push(x);
    }

    let starting_cards = {
        let mut card_ids = Vec::new();
        for (_, PackAsset { starting_cards, .. }) in packs.iter() {
            for card_handle in starting_cards {
                card_ids.push(CardId(
                    sorted_cards
                        .iter()
                        .enumerate()
                        .find(|(_, (asset_id, _))| *asset_id == card_handle.id())
                        .unwrap()
                        .0 as u32,
                ));
            }
        }

        card_ids.into_boxed_slice()
    };

    BackendAssetData {
        cards: logical_cards.into_boxed_slice(),
        markets: logical_markets.into_boxed_slice(),
        initial_player_inventory: starting_cards,
    }
}
