mod audio;
mod big_bird;
mod camera;
mod clouds;
mod debug;
mod input;
mod ship;
mod ui;

use avian3d::prelude::*;
use bevy::audio::{AudioPlugin, Volume};
use bevy::light::atmosphere::ScatteringMedium;
use bevy::light::{Atmosphere, CascadeShadowConfigBuilder, VolumetricLight};
use bevy::mesh::{SphereKind, SphereMeshBuilder};
use bevy::prelude::*;
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use bevy_hanabi::HanabiPlugin;
use bevy_rand::prelude::*;
use big_space::prelude::*;
use shadow_rs::shadow;

shadow!(build_info);

fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(120.0))
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            EmbeddedAssetPlugin {
                mode: PluginMode::ReplaceDefault,
            },
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Birdy Flight Sim".into(),
                        canvas: Some("#bevy".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),

                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: GlobalVolume::new(Volume::Linear(0.1)),
                    ..default()
                })
                .disable::<TransformPlugin>(),
            BigSpaceDefaultPlugins,
            PhysicsPlugins::default(),
            big_bird::BigSpaceAvianSyncPlugin,
            big_bird::BigSpaceHanabiSyncPlugin,
            EntropyPlugin::<WyRand>::default(),
            HanabiPlugin,
            audio::ProceduralAudioPlugin,
            input::InputPlugin,
            ship::ShipPlugin,
            ui::HudPlugin,
            camera::ChaseCameraPlugin,
            clouds::CloudsPlugin,
            debug::DebugPlugin,
        ))
        .add_systems(PreStartup, set_global_default_font)
        .add_systems(Startup, (setup_space, setup_celestial_bodies).chain())
        .run();
}

/// Override default font with JetBrains Mono
fn set_global_default_font(mut fonts: ResMut<Assets<Font>>) {
    const FONT_DATA: &[u8] = include_bytes!("../assets/game/fonts/JetBrainsMono-Regular.ttf");

    let font = Font::from_bytes(FONT_DATA.to_vec());

    // Overwrite Bevy's default font handle
    fonts
        .insert(&Handle::default(), font)
        .expect("Failed to insert font");
}

fn setup_space(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        BigSpace
        Grid
        Visibility::Visible
        Children [
            camera::camera_scene(),

            (
                #Sun
                DirectionalLight {
                    illuminance: 120_000.0,
                    shadow_maps_enabled: true,
                }
                VolumetricLight
                template_value(CascadeShadowConfigBuilder {
                    num_cascades: 4,
                    minimum_distance: 0.5,
                    maximum_distance: 10_000.0,
                    first_cascade_far_bound: 100.0,
                        ..default()
                }
                .build())
                CellCoord
                template_value(Transform::from_xyz(10_000.0, 15_000.0, 10_000.0).looking_at(Vec3::ZERO, Vec3::Y))
            )
        ]
    });
}

fn setup_celestial_bodies(
    mut commands: Commands,
    space: Single<Entity, With<BigSpace>>,
    mut mediums: ResMut<Assets<ScatteringMedium>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Atmosphere
    let earth_medium = mediums.add(ScatteringMedium::earth(512, 512));
    let earth_atmosphere = Atmosphere::earth(earth_medium);

    let scale = 0.1;
    let planet_radius = earth_atmosphere.inner_radius * scale;

    commands.entity(*space).with_children(|parent| {
        parent.spawn((
            Name::new("PlanetAtmosphere"),
            earth_atmosphere,
            CellCoord::default(),
            Transform::from_scale(Vec3::splat(scale)).with_translation(Vec3::new(
                0.0,
                -planet_radius,
                0.0,
            )),
        ));

        // Ground Plane
        parent.spawn((
            Name::new("Planet"),
            Mesh3d(meshes.add(Mesh::from(SphereMeshBuilder::new(
                planet_radius,
                SphereKind::Ico { subdivisions: 20 },
            )))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.1, 0.35, 0.15),
                perceptual_roughness: 0.9,
                ..default()
            })),
            CellCoord::default(),
            Transform::from_xyz(0.0, -planet_radius, 0.0),
            RigidBody::Static,
            Collider::sphere(planet_radius),
        ));
    });
}
