use crate::ship::FlightModel;
use avian3d::debug_render::PhysicsGizmos;
use avian3d::prelude::*;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub(crate) struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        let debug_mode = input_toggle_active(false, KeyCode::F12);

        app.add_plugins((
            WorldInspectorPlugin::new().run_if(debug_mode.clone()),
            PhysicsDebugPlugin,
        ))
        .insert_gizmo_config(
            PhysicsGizmos::default(),
            GizmoConfig {
                enabled: false,
                ..default()
            },
        )
        .add_systems(
            Update,
            (
                draw_flight_vectors.run_if(debug_mode),
                toggle_physics_debug_gizmos,
            ),
        );
    }
}

pub fn draw_flight_vectors(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &LinearVelocity), With<FlightModel>>,
) {
    let scale = 0.5; // Adjust visual length of vectors

    for (transform, linear_velocity) in &query {
        let position = transform.translation;
        let actual_vel = linear_velocity.0; // Total velocity (Vec3)

        // Skip if not moving
        if actual_vel.length_squared() < 0.001 {
            continue;
        }

        // 1. Get the direction the aircraft nose is pointing
        let forward_dir = *transform.forward(); // Vec3

        // 2. Calculate speed along the aircraft's longitudinal axis
        // (Dot product projects actual velocity onto the forward direction)
        let forward_speed = actual_vel.dot(forward_dir);
        let forward_vel = forward_dir * forward_speed;

        // --- DRAW GIZMOS ---

        // A. Actual Trajectory / Velocity Vector (Green)
        gizmos.arrow(
            position,
            position + (actual_vel * scale),
            Color::srgb(0.0, 1.0, 0.0),
        );

        // B. Forward Velocity / Nose Vector (Yellow - Ignores Lift & Gravity)
        gizmos.arrow(
            position,
            position + (forward_vel * scale),
            Color::srgb(1.0, 1.0, 0.0),
        );

        // C. Lift / Gravity Delta Vector (Red - Shows where forces pull the plane)
        // Drawn from the tip of the Yellow vector to the tip of the Green vector
        gizmos.arrow(
            position + (forward_vel * scale),
            position + (actual_vel * scale),
            Color::srgb(1.0, 0.0, 0.0),
        );
    }
}

pub fn toggle_physics_debug_gizmos(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    if keyboard.just_pressed(KeyCode::F12) {
        let (config, _) = config_store.config_mut::<PhysicsGizmos>();
        config.enabled = !config.enabled;
    }
}
