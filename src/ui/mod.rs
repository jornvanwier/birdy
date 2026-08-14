use crate::build_info;
use bevy::camera::visibility::RenderLayers;
use bevy::gizmos::config::GizmoConfigStore;
use bevy::prelude::*;

mod attitude;
mod telemetry;

pub use attitude::HudGizmos;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<HudGizmos>() // 1. Register group
            .add_systems(
                Startup,
                (
                    spawn_camera,
                    setup_hud_gizmo_config, // 2. Assign ONLY HudGizmos to Layer 1
                    telemetry::setup_telemetry,
                    setup_version,
                ),
            )
            .add_systems(
                Update,
                (telemetry::update_telemetry, attitude::draw_attitude_hud),
            );
    }
}

fn setup_hud_gizmo_config(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<HudGizmos>();
    config.render_layers = RenderLayers::layer(1);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 10,
            ..default()
        },
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
