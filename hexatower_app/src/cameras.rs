use bevy::{camera::Viewport, prelude::*, window::WindowResized};

use crate::AppState;

pub struct CameraManagementPlugin;

impl Plugin for CameraManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (resize_3d_viewport, move_3d_cam).run_if(in_state(AppState::InGame)),
        );
        app.add_systems(
            OnEnter(AppState::InGame),
            (spawn_in_game_cameras_and_lights, initial_resize_event),
        );
        app.add_systems(OnEnter(AppState::MainMenu), spawn_main_menu_cam);
    }
}

#[derive(Debug, Component)]
struct InGameCam3d;
#[derive(Debug, Component)]
pub struct InGameCam2d;

fn resize_3d_viewport(
    windows: Query<&Window>,
    mut resize_events: MessageReader<WindowResized>,
    mut cam_3d: Single<&mut Camera, With<InGameCam3d>>,
) {
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();

        cam_3d.viewport = Some(Viewport {
            physical_position: UVec2 {
                x: window.physical_width() / 2,
                y: 0,
            },
            physical_size: UVec2 {
                x: window.physical_width() / 2,
                y: window.physical_height(),
            },
            ..default()
        });
    }
}

// this system just emits an event with the same window info as it starts with to get the "resize_3d_viewport" function to run without the player needing to resize the window.
fn initial_resize_event(
    mut writer: MessageWriter<WindowResized>,
    windows: Query<(&Window, Entity)>,
) {
    for (window, window_entity) in windows {
        writer.write(WindowResized {
            window: window_entity,
            width: window.width(),
            height: window.height(),
        });
    }
}

struct Cam3dControls {
    climb_angle: f32,
    rotation_about_vertical: f32,
    x: f32,
    z: f32,
    radius: f32,
}

impl From<Cam3dControls> for Transform {
    fn from(value: Cam3dControls) -> Self {
        let translation = Vec3 {
            x: value.x
                + value.radius
                    * f32::cos(value.climb_angle)
                    * f32::cos(value.rotation_about_vertical),
            y: value.radius * f32::sin(value.climb_angle),
            z: value.z
                + value.radius
                    * f32::cos(value.climb_angle)
                    * f32::sin(value.rotation_about_vertical),
        };

        Transform::from_translation(translation).looking_at(
            Vec3 {
                x: value.x,
                y: 0.0,
                z: value.z,
            },
            Vec3::Y,
        )
    }
}

pub fn spawn_in_game_cameras_and_lights(mut commands: Commands) {
    commands.spawn((
        Transform::from(Cam3dControls {
            climb_angle: 1.0,
            rotation_about_vertical: 1.0,
            x: 2.5,
            z: 5.0,
            radius: 3.0,
        }),
        Camera3d::default(),
        Camera {
            order: 0,
            ..Default::default()
        },
        InGameCam3d,
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 20,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
        InGameCam2d,
    ));

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

#[derive(Debug, Component)]
pub struct MainMenuCam;

pub fn spawn_main_menu_cam(mut commands: Commands) {
    commands.spawn((Camera2d, MainMenuCam, DespawnOnEnter(AppState::InGame)));
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
