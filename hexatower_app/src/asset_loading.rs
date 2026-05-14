use bevy::{asset::AssetLoader, ecs::schedule::ScheduleLabel, prelude::*};
use core_game_logic::cards::{CardFunction, CardId, LogicalCard};
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
    functionality: CardFunction,
}

#[derive(Debug)]
struct CardPrice(u32);

#[derive(Debug, Asset, TypePath)]
struct IntermediateMarket {
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

struct BackendAssetData {
    cards: Box<LogicalCard>,
}

fn sort_cards(
    full_cards: Res<Assets<IntermediateCard>>,
    mut commands: Commands,
) -> BackendAssetData {
    let sorted_cards = {
        let mut pre_sorted = full_cards.iter().collect::<Vec<_>>();
        pre_sorted.sort_by_key(|(_, card_data)| card_data.name.clone());
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

    todo!()
}
