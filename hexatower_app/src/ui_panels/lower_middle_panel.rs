use bevy::{color::palettes, prelude::*};
use core_game_logic::players::{Inventory, PlayerDirectory, PlayerId};

use crate::{
    functional_assets::{LogicalWorld, SetUpBoard},
    lower_middle_panel,
};

pub struct MiddlePanelPlugin;

impl Plugin for MiddlePanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SetUpBoard, spawn_middle_ui_node);
    }
}

fn render_player_inventory(
    player: PlayerId,
    commands: &mut Commands,
    world: &LogicalWorld,
) -> Result<(), BevyError> {
    let player_inventory = world
        .0
        .get::<Inventory>(world.0.resource::<PlayerDirectory>().get_player(player)?)
        .ok_or("logical player had no inventory")?;

    for (index, card) in player_inventory.all_cards().iter().enumerate() {
        // render them to the screen.
    }

    Ok(())
}

fn spawn_middle_ui_node(mut commands: Commands) {
    commands.spawn((
        Node {
            height: Val::Percent(18.0),
            width: Val::Percent(35.0),
            align_self: AlignSelf::End,
            justify_self: JustifySelf::Center,
            ..default()
        },
        BackgroundColor(palettes::tailwind::SLATE_400.into()),
    ));

    commands.spawn((
        Node {
            height: Val::Percent(18.0),
            width: Val::Percent(35.0),
            align_self: AlignSelf::End,
            justify_self: JustifySelf::Start,
            ..default()
        },
        BackgroundColor(palettes::tailwind::AMBER_700.into()),
    ));
}
