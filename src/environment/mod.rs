use crate::camera;
use crate::environment::air_density::AtmosphereProperties;
pub(crate) use crate::environment::gravity::{LocalGravity, apply_local_gravity};
use bevy::light::{CascadeShadowConfigBuilder, VolumetricLight};
use bevy::prelude::*;
use big_space::prelude::*;

pub mod air_density;
pub mod celestial_body;
pub mod clouds;
pub mod gravity;

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
                    update_ship_environment,
                    apply_local_gravity,
                )
                    .chain()
                    .in_set(EnvironmentSet),
            );
    }
}

/// Measured altitude above the sea level of the closest celestial body.
#[derive(Component, Default, Copy, Clone, Debug, Deref, DerefMut)]
pub struct Altitude(pub f32);

/// Directional reference frame relative to the nearest celestial body.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PlanetaryFrame {
    /// Orientation of the local ground (East = +X, Up = +Y, South = +Z / North = -Z)
    pub rotation: Quat,
}

/// Composite bundle / required components for any entity interacting with planetary environments.
#[derive(Component, Default, Copy, Clone, Debug)]
#[require(ClosestBody, Altitude, LocalAirDensity, LocalGravity, PlanetaryFrame)]
pub struct CelestialBodyEnvironment;

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

pub fn update_ship_environment(
    grid: Single<&Grid, With<BigSpace>>,
    bodies: Query<(
        &CelestialBody,
        Option<&AtmosphereProperties>,
        CellTransformReadOnly,
    )>,
    mut ships: Query<(
        &ClosestBody,
        CellTransformReadOnly,
        &mut Altitude,
        &mut LocalAirDensity,
        &mut LocalGravity,
        &mut PlanetaryFrame,
    )>,
) {
    for (closest_body, ship_transform, mut altitude, mut density, mut gravity, mut frame) in &mut ships {
        // 1. Deep Space Fallbacks
        let Some(body_entity) = closest_body.0 else {
            altitude.0 = f32::INFINITY;
            density.0 = 0.0;
            gravity.0 = Vec3::ZERO;
            frame.rotation = Quat::IDENTITY;
            continue;
        };

        let Ok((body, maybe_atmosphere, body_transform)) = bodies.get(body_entity) else {
            altitude.0 = f32::INFINITY;
            density.0 = 0.0;
            gravity.0 = Vec3::ZERO;
            frame.rotation = Quat::IDENTITY;
            continue;
        };

        // 2. Compute Relative Vector & Distance
        let ship_pos = grid.grid_position_double(ship_transform.cell, ship_transform.transform);
        let body_pos = grid.grid_position_double(body_transform.cell, body_transform.transform);

        let delta = (body_pos - ship_pos).as_vec3();
        let distance = delta.length();
        let current_altitude = distance - body.radius;

        // 3. Set Altitude & Density
        altitude.0 = current_altitude;
        density.0 = maybe_atmosphere
            .map(|a| a.density_at_altitude(current_altitude))
            .unwrap_or(0.0);

        // 4. Set Gravity & Planetary Surface Frame
        if distance > 1e-4 {
            let gravity_dir = delta / distance; // Points toward planet center
            gravity.0 = gravity_dir * body.gravity_at_distance(distance);

            let up = -gravity_dir;
            // Project world +Y (planet pole) onto local horizon plane to find North
            let north = (Vec3::Y - up * Vec3::Y.dot(up)).normalize_or_zero();
            let north = if north.length_squared() > 0.0 { north } else { Vec3::Z };
            let east = north.cross(up);

            // Construct local basis: +X = East, +Y = Up, +Z = South (-North)
            frame.rotation = Quat::from_mat3(&Mat3::from_cols(east, up, -north));
        } else {
            gravity.0 = Vec3::ZERO;
            frame.rotation = Quat::IDENTITY;
        }
    }
}