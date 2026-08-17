mod camera;
mod debug;
mod input;
pub mod ship;
mod ui;

use avian3d::prelude::*;
use bevy::light::Atmosphere;
use bevy::light::atmosphere::ScatteringMedium;
use bevy::prelude::*;
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use shadow_rs::shadow;

shadow!(build_info);

fn main() {
    App::new()
        // ClearColor black is recommended for physically-based sky rendering
        .insert_resource(ClearColor(Color::BLACK))
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
            PhysicsPlugins::default(),
            input::InputPlugin,
            ship::ShipPlugin,
            ui::HudPlugin,
            camera::ChaseCameraPlugin,
            debug::DebugPlugin,
        ))
        .add_systems(Startup, setup_atmosphere_and_scene)
        .run();
}

fn setup_atmosphere_and_scene(
    mut commands: Commands,
    mut mediums: ResMut<Assets<ScatteringMedium>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1. Create standard Earth-like scattering medium
    let earth_medium = mediums.add(ScatteringMedium::earth(512, 512));
    let earth_atmosphere = Atmosphere::earth(earth_medium);
    let planet_radius = earth_atmosphere.inner_radius;

    commands.spawn((
        Name::new("PlanetAtmosphere"),
        earth_atmosphere,
        Transform::from_xyz(0.0, -planet_radius, 0.0),
    ));

    // 2. High-intensity Sun (Directional Light)
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 120_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(10_000.0, 15_000.0, 10_000.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 3. Ground Plane
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(50_000.0, 50_000.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.35, 0.15),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));

    // 4. Cloud Layer
    commands.spawn((
        Name::new("CloudDeck"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::new(100_000.0, 100_000.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.6),
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, 2_000.0, 0.0),
    ));
}
