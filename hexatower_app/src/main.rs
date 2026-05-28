use std::f32::consts::PI;

use bevy::{post_process::bloom::Bloom, prelude::*};

use crate::{
    functional_assets::{GameCreationSettings, SetUpBoard, StartupPlugin},
    inputs_interface::InputInterfacePlugin,
    vis_tiles::{BoardSize, VisTilesPlugin},
};

mod functional_assets;
mod inputs_interface;
mod vis_tiles;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            StartupPlugin,
            VisTilesPlugin,
            InputInterfacePlugin,
        ))
        .add_systems(SetUpBoard, (cam_3d, lights))
        .add_systems(Update, move_3d_cam)
        .run();
}

fn cam_3d(settings: Res<GameCreationSettings>, mut commands: Commands) {
    let distance = match settings.board_size {
        BoardSize::Small => 16.0,
        BoardSize::Standard => 22.0,
        BoardSize::Large => 25.0,
        BoardSize::ExtraLarge => 38.0,
    };

    let angle = match settings.board_size {
        BoardSize::Small => PI / 3.0,
        BoardSize::Standard => PI / 2.9,
        BoardSize::Large => PI / 2.8,
        BoardSize::ExtraLarge => PI / 5.0,
    };

    commands.spawn((
        Transform::default()
            .with_translation(
                Vec3::default()
                    .with_z(angle.cos() * distance)
                    .with_y(angle.sin() * distance),
            )
            .looking_at(Vec3::ZERO, Vec3::Y),
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
