use crate::camera;
use crate::environment::air_density::calculate_local_air_density;
use bevy::light::{CascadeShadowConfigBuilder, VolumetricLight};
use bevy::prelude::*;
use big_space::prelude::*;

pub mod air_density;
pub mod celestial_body;
pub mod clouds;

use crate::environment::celestial_body::determine_closest_celestial_body;
pub use air_density::LocalAirDensity;
pub use celestial_body::{CelestialBody, ClosestBody};

pub struct EnvironmentPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct EnvironmentSet;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(clouds::CloudsPlugin)
            .add_systems(PreStartup, setup_space)
            .add_systems(Startup, celestial_body::setup_celestial_bodies)
            .add_systems(
                FixedUpdate,
                (
                    determine_closest_celestial_body,
                    calculate_local_air_density,
                )
                    .chain()
                    .in_set(EnvironmentSet),
            );
    }
}

pub fn setup_space(mut commands: Commands) {
    info!("Setting up space");
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
