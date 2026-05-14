use bevy::{asset::AssetLoader, ecs::schedule::ScheduleLabel, prelude::*};
use core_game_logic::cards::CardId;
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

        app.add_systems(AssetsFinishedLoading, syst_sort_cards);
    }
}

#[derive(Debug, Resource)]
struct ActivePackAssets(Box<[Handle<PackAsset>]>);

#[derive(Debug, Asset, TypePath)]
struct PackAsset {
    starting_cards: Box<[Handle<CardAsset>]>,
    starting_markets: Box<[Handle<MarketAsset>]>,
}

#[derive(Debug, Asset, TypePath, Clone)]
struct CardAsset {
    name: String,
}
#[derive(Debug)]
struct CardPrice(u32);

#[derive(Debug, Asset, TypePath)]
struct MarketAsset {
    offers: [(Handle<CardAsset>, CardPrice); 3],
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

#[derive(Debug, Resource)]
struct SortedCards(Box<[Handle<CardAsset>]>);

impl SortedCards {
    pub fn get_id(&self, handle: Handle<CardAsset>) -> CardId {
        CardId(
            self.0
                .iter()
                .enumerate()
                .find(|(_, card)| **card == handle)
                .expect("Had a handle to a card asset which was not in the list of sorted cards.")
                .0 as u32,
        )
    }
}

fn syst_sort_cards(
    mut card_events: MessageReader<AssetEvent<CardAsset>>,
    mut cards: ResMut<Assets<CardAsset>>,
    mut commands: Commands,
) {
    let mut list_to_sort = card_events
        .read()
        .filter_map(|event| match event {
            AssetEvent::LoadedWithDependencies { id } => Some((
                cards.get_strong_handle(*id).unwrap(),
                cards.get(*id).unwrap().clone(),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();

    list_to_sort.sort_by_key(|(_, card_data)| card_data.name.clone());

    commands.insert_resource(SortedCards(
        list_to_sort
            .iter()
            .map(|(handle, _)| handle.clone())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    ));
}
