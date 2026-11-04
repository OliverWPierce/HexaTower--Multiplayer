use bevy::color::palettes::tailwind::*;
pub use bevy::prelude::*;
use core_game_logic::{players::PlayerData, requests::ActionEffect};

use crate::{
    functional_assets::PlayerNames, inputs_interface::EffectToDisplay,
    vis_effect_reactions::DespawnTimer,
};

pub fn banners(
    effect: Res<EffectToDisplay>,
    mut commands: Commands,
    names: Res<PlayerNames>,
    player_colors: Res<PlayerData<Color>>,
) {
    let banner_info: (Color, Color, String) = match effect.0 {
        ActionEffect::BeganTurn(player) => (
            player_colors.get(player).darker(0.5),
            *player_colors.get(player),
            format!("Beginning {}'s turn!", names.get(player)),
        ),
        ActionEffect::EndedTurn(player) => (
            player_colors.get(player).darker(0.5),
            *player_colors.get(player),
            format!("{} ended their turn!", names.get(player)),
        ),
        ActionEffect::PlayerDied(player) => (
            player_colors.get(player).darker(0.5),
            *player_colors.get(player),
            format!("{} died!", names.get(player)),
        ),
        ActionEffect::GameOver {
            winner: Some(player),
        } => (
            player_colors.get(player).darker(0.5),
            *player_colors.get(player),
            format!("{} won!", names.get(player)),
        ),
        ActionEffect::GameOver { winner: None } => (
            Color::Srgba(ZINC_900),
            Color::Srgba(ZINC_800),
            "Draw!".into(),
        ),
        _ => return,
    };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        DespawnTimer(Timer::from_seconds(4.0, TimerMode::Once)),
        children![(
            Node {
                height: Val::Percent(5.0),
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(banner_info.1),
            children![(
                Text::new(banner_info.2),
                TextFont::from_font_size(FontSize::Vh(3.0))
            )]
        )],
    ));
}
