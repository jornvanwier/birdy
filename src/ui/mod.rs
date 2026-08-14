use crate::build_info;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

mod telemetry;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_camera, telemetry::setup_telemetry, setup_version),
        )
        .add_systems(Update, (telemetry::update_telemetry,));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            // Renders UI after main scene (0) and orb (1)
            order: 10,
            ..default()
        },
        // Directs all 2D UI nodes here
        IsDefaultUiCamera,
        RenderLayers::layer(1),
    ));
}

pub fn setup_version(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Node {
            position_type: PositionType::Absolute,
            bottom: px(3.0),
            right: px(3.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(4.0),
            padding: UiRect::all(px(8.0)),
        }
        Children [
            (
                Text(format!("Commit: {}", build_info::SHORT_COMMIT))
                TextFont { font_size: px(8.0) }
                TextColor(Color::WHITE)
            ),
            (
                Text(format!("Build time: {}", build_info::BUILD_TIME))
                TextFont { font_size: px(8.0) }
                TextColor(Color::WHITE)
            ),
        ]
    });
}
