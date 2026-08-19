mod camera;
mod clouds;
mod debug;
mod input;
mod ship;
mod ui;

use avian3d::prelude::*;
use bevy::light::atmosphere::ScatteringMedium;
use bevy::light::{Atmosphere, CascadeShadowConfigBuilder, VolumetricLight};
use bevy::prelude::*;
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use bevy_hanabi::HanabiPlugin;
use bevy_rand::prelude::*;
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
            EntropyPlugin::<WyRand>::default(),
            PhysicsPlugins::default(),
            HanabiPlugin,
            input::InputPlugin,
            ship::ShipPlugin,
            ui::HudPlugin,
            camera::ChaseCameraPlugin,
            clouds::CloudsPlugin,
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

    // 2. Sun with extended shadow cascades
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 120_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        VolumetricLight,
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 0.5,
            maximum_distance: 10_000.0,
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
}