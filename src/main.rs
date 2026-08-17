mod camera;
mod debug;
mod input;
mod ship;
mod ui;

use avian3d::prelude::*;
use bevy::light::atmosphere::ScatteringMedium;
use bevy::light::{Atmosphere, CascadeShadowConfigBuilder, FogVolume, VolumetricLight};
use bevy::prelude::*;
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use shadow_rs::shadow;

shadow!(build_info);

fn main() {
    App::new()
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
    // 1. Atmosphere
    let earth_medium = mediums.add(ScatteringMedium::earth(512, 512));
    let earth_atmosphere = Atmosphere::earth(earth_medium);
    let planet_radius = earth_atmosphere.inner_radius;

    commands.spawn((
        Name::new("PlanetAtmosphere"),
        earth_atmosphere,
        Transform::from_xyz(0.0, -planet_radius, 0.0),
    ));

    // 2. High-intensity Sun with extended shadow cascades
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 120_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        VolumetricLight,
        // Extend shadow distance so high/distant clouds receive sunlight
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 0.5,
            maximum_distance: 10_000.0, // 10 km shadow distance
            first_cascade_far_bound: 100.0,
            ..default()
        }
        .build(),
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

    // 4. Volumetric Clouds (Spawn multiple distinct cloud patches or bank)
    // In src/main.rs
    let cloud_positions = [
        Vec3::new(0.0, 800.0, -2_000.0),
        Vec3::new(2_500.0, 1_000.0, -4_000.0),
        Vec3::new(-2_000.0, 700.0, -3_000.0),
    ];

    for (i, pos) in cloud_positions.into_iter().enumerate() {
        commands.spawn((
            Name::new(format!("VolumetricCloud_{i}")),
            FogVolume {
                fog_color: Color::srgb(1.0, 1.0, 1.0),
                // Calibrated for ~2.5 km volume radius (peak brightness range: 0.0004 - 0.0008)
                density_factor: 0.0006,
                scattering: 0.3,
                absorption: 0.3,            // Pure scattering (no dark soot)
                scattering_asymmetry: 0.5, // Near-isotropic: bright white from any viewing angle
                ..default()
            },
            Transform::from_translation(pos).with_scale(Vec3::new(2_500.0, 300.0, 2_500.0)),
        ));
    }
}
