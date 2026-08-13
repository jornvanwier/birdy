mod input;
pub mod ship;

use crate::input::Action;
use crate::ship::{FlightModel, ShipPlugin};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_plugins(ShipPlugin)
        .add_systems(Startup, scene.spawn())
        .add_systems(Update, draw_flight_vectors)
        .run();
}

fn draw_flight_vectors(mut gizmos: Gizmos, query: Query<(&Transform, &LinearVelocity), With<FlightModel>>) {
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

/// set up a simple 3D scene
fn scene() -> impl SceneList {
    let planet_radius = 1000.;
    bsn_list! [
        (
            #Planet
            Mesh3d(asset_value(Sphere::new(planet_radius)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb(0., 0.9, 0.1)))
            Transform {
                translation: Vec3::new(0., -planet_radius, 0.),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            }
            template_value(RigidBody::Static)
            Collider::sphere(planet_radius)
        ),
        (
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
            Transform::from_xyz(-1.0, 10.0, 0.0)
            template_value(RigidBody::Dynamic)
            Collider::cuboid(1.0, 1.0, 1.0)
            Mass(5.0)
            LinearDamping(0.5)
        ),
        (
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(30, 144, 255)))
            Transform::from_xyz(1.0, 10.0, 0.0)
            template_value(RigidBody::Dynamic)
            Collider::cuboid(1.0, 1.0, 1.0)
            Mass(20.0)
            LinearDamping(0.5)
        ),
        (
            DirectionalLight {
                 shadow_maps_enabled: true,
             }
            template_value(Transform::from_xyz(4000.0, 8000.0, 4000.0).looking_at(Vec3::ZERO, Vec3::Y))
        )
    ]
}
