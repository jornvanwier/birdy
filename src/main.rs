mod debug;
mod input;
pub mod ship;
mod ui;

use crate::ship::ShipPlugin;
use crate::ui::HudPlugin;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use shadow_rs::shadow;

shadow!(build_info);

fn main() {
    App::new()
        .add_plugins((
            EmbeddedAssetPlugin {
                mode: PluginMode::ReplaceDefault,
            },
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Birdy Flight Sim".into(),
                    canvas: Some("#bevy".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin::default(),
            PhysicsPlugins::default(),
            input::InputPlugin,
            ShipPlugin,
            HudPlugin,
            debug::DebugPlugin,
        ))
        .add_systems(Startup, scene.spawn())
        .run();
}

/// set up a simple 3D scene
fn scene() -> impl SceneList {
    let planet_radius = 1000.;
    bsn_list! [
        (
            #Planet
            Mesh3d(asset_value(Sphere::new(planet_radius)))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.1, 0.1, 0.8)))
            Transform {
                translation: Vec3::new(500., planet_radius + 1000., 500.),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            }
            template_value(RigidBody::Static)
            Collider::sphere(planet_radius)
        ),
        (
            #Ground
            Mesh3d(asset_value(Plane3d::new(Vec3::Y, Vec2::new(10_000., 10_000.))))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.1, 0.4, 0.2)))
            Transform::from_translation(Vec3::ZERO)
            template_value(RigidBody::Static)
            Collider::half_space(Vec3::Y)
        ),
        (
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
            Transform::from_xyz(-1.0, 10.0, 0.0)
            template_value(RigidBody::Dynamic)
            Collider::cuboid(1.0, 1.0, 1.0)
            Mass(5.0)
            LinearDamping(0.5)
        ),
        (
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb_u8(30, 144, 255)))
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
