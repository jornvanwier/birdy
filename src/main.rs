pub mod ship;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(PhysicsDebugPlugin)
        .add_systems(Startup, scene.spawn())
        .add_systems(Startup, ship::spawn_ship)
        .run();
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
            PointLight {
                shadow_maps_enabled: true,
            }
            Transform::from_xyz(4.0, 8.0, 4.0)
        )
    ]
}
