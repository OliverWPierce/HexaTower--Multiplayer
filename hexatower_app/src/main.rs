use bevy::{post_process::bloom::Bloom, prelude::*};
use bevy_obj::ObjPlugin;
use bevy_renet::{
    RenetClientPlugin, RenetServerPlugin,
    netcode::{NetcodeClientPlugin, NetcodeServerPlugin},
};
use core_game_logic::players::PlayerId;

use crate::{
    functional_assets::SetUpBoard, inputs_interface::InputInterfacePlugin,
    main_menu::MainMenuAndLobbyPluggin, ui_panels::UiPanelsPlugin,
    vis_effect_reactions::VisEffectReactions, vis_markets::VisMarketsPlugin,
    vis_pieces::VisPiecesPlugin, vis_tiles::VisTilesPlugin,
};

const VERSION_NUMBER: u64 = 0;

mod functional_assets;
mod inputs_interface;
mod main_menu;
mod ui_panels;
mod vis_effect_reactions;
mod vis_markets;
mod vis_pieces;
mod vis_tiles;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            VisTilesPlugin,
            InputInterfacePlugin,
            UiPanelsPlugin,
            VisPiecesPlugin,
            VisMarketsPlugin,
            ObjPlugin,
            VisEffectReactions,
            MainMenuAndLobbyPluggin,
            RenetClientPlugin,
            RenetServerPlugin,
            NetcodeClientPlugin,
            NetcodeServerPlugin,
        ))
        .init_state::<AppState>()
        .add_systems(SetUpBoard, (cam_3d, lights))
        .add_systems(Update, move_3d_cam.run_if(in_state(AppState::InGame)))
        .run();
}

/// The player that the operator of the device is representing. A spectator of a match would be Option::None, since they are not acting as a player, just a spectator. However, they will have a display player, so that the game can know which player's inventory and stats to display to the spectator.
#[derive(Debug, Resource)]
pub struct OperatingPlayer(PlayerId);

#[derive(Debug, States, Clone, Copy, Default, Eq, Hash, PartialEq)]
pub enum AppState {
    #[default]
    MainMenu,
    ParametersScreen,
    PreGame,
    InGame,
}

fn cam_3d(mut commands: Commands) {
    let desired_transform = Transform::default()
        .with_translation(Vec3 {
            x: 0.0,
            z: 2.0,
            y: 20.0,
        })
        .looking_at(Vec3::ZERO, Dir3::Y);

    commands.spawn((
        // Transform::default()
        //     .with_translation(
        //         Vec3::default()
        //             .with_z(angle.cos() * distance)
        //             .with_y(angle.sin() * distance),
        //     ),
        desired_transform,
        Camera3d::default(),
        Bloom::NATURAL,
    ));
}

fn lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 6000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::default()
            .with_translation(vec3(100.0, 200.0, 300.0))
            .looking_at(Vec3::ZERO, Dir3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 120000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::default().with_translation(vec3(0.0, 3.0, 0.0)),
    ));
}

fn move_3d_cam(
    inputs: Res<ButtonInput<KeyCode>>,
    camera: Single<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
) {
    const VERT_SPEED: f32 = 1.2;
    const HORIZONTAL_SPEED: f32 = 0.7;

    camera.into_inner().translation += Vec3 {
        x: (inputs.pressed(KeyCode::KeyD) as i8 - inputs.pressed(KeyCode::KeyA) as i8) as f32
            * HORIZONTAL_SPEED,
        y: (inputs.pressed(KeyCode::Space) as i8 - inputs.pressed(KeyCode::ShiftLeft) as i8) as f32
            * VERT_SPEED,
        z: (inputs.pressed(KeyCode::KeyS) as i8 - inputs.pressed(KeyCode::KeyW) as i8) as f32
            * HORIZONTAL_SPEED,
    } * time.delta_secs();
}
