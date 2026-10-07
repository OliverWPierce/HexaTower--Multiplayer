use bevy::{camera::Viewport, prelude::*, window::WindowResized};

use crate::AppState;

pub struct CameraManagementPlugin;

impl Plugin for CameraManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            resize_3d_viewport.run_if(in_state(AppState::InGame)),
        );
        app.add_systems(
            OnEnter(AppState::InGame),
            (spawn_in_game_cameras, initial_resize_event),
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

pub fn spawn_in_game_cameras(mut commands: Commands) {
    let distance: f32 = 16.0;

    let angle: f32 = 1.3;

    commands.spawn((
        Transform::default()
            .with_translation(
                Vec3::default()
                    .with_z(angle.cos() * distance)
                    .with_y(angle.cos() * distance),
            )
            .looking_at(Vec3::ZERO, Vec3::Y),
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
}
#[derive(Debug, Component)]
pub struct MainMenuCam;

pub fn spawn_main_menu_cam(mut commands: Commands) {
    commands.spawn((Camera2d, MainMenuCam, DespawnOnEnter(AppState::InGame)));
}
