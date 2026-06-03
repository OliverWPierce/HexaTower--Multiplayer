use std::f32::consts::PI;

use bevy::{math::VectorSpace, post_process::bloom::Bloom, prelude::*};
use core_game_logic::players::PlayerId;

use crate::{
    functional_assets::{GameCreationSettings, SetUpBoard, StartupPlugin},
    inputs_interface::InputInterfacePlugin,
    ui_panels::UiPanelsPlugin,
    vis_tiles::{BoardSize, VisTilesPlugin},
};

mod functional_assets;
mod inputs_interface;
mod ui_panels;
mod vis_tiles;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            StartupPlugin,
            VisTilesPlugin,
            InputInterfacePlugin,
            UiPanelsPlugin,
        ))
        .add_systems(SetUpBoard, (cam_3d, lights))
        .add_systems(Update, move_3d_cam)
        .run();
}

/// The player whom the screen should be rendered for. Most likely, this is the player using this device. However, this could also be used for spectator modes.
#[derive(Debug, Resource)]
pub struct DisplayPlayer(pub PlayerId);

/// The player that the operator of the device is representing. A spectator of a match would be Option::None, since they are not acting as a player, just a spectator. However, they will have a display player, so that the game can know which player's inventory and stats to display to the spectator.
#[derive(Debug, Resource)]
pub struct OperatingPlayer(pub Option<PlayerId>);

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
            shadows_enabled: true,
            ..default()
        },
        Transform::default()
            .with_translation(vec3(100.0, 200.0, 300.0))
            .looking_at(Vec3::ZERO, Dir3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 120000.0,
            shadows_enabled: true,
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
