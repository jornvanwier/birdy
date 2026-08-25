use crate::environment::celestial_body::{CelestialBody, ClosestBody};
use bevy::light::Atmosphere;
use bevy::light::atmosphere::ScatteringMedium;
use bevy::prelude::*;
use big_space::prelude::*;

/// Standard scalar dynamic pressure (q = 0.5 * rho * v^2) in Pascals (N/m^2).
/// Used for continuous airflow across lifting surfaces.
#[inline]
pub fn calculate_scalar_dynamic_pressure(air_density: f32, speed: f32) -> f32 {
    0.5 * air_density * speed.powi(2)
}

/// Signed, component-wise dynamic pressure per body axis (q_i = 0.5 * rho * v_i * |v_i|).
/// Preserves directional sign along each local axis for bluff-body cross-flow drag.
#[inline]
pub fn calculate_component_dynamic_pressure(air_density: f32, local_velocity: Vec3) -> Vec3 {
    0.5 * air_density * local_velocity * local_velocity.abs()
}

#[derive(Component, Reflect, Clone, Copy, Debug)]
pub struct AtmosphereProperties {
    /// Density at sea level (h = 0) in kg/m^3 (e.g. Earth = 1.225)
    sea_level_density: f32,
    /// Scale height in meters (e.g. Earth ~ 8500.0)
    scale_height: f32,
    /// Total atmosphere ceiling above surface where density cuts off to 0.0
    atmosphere_height: f32,
}

impl AtmosphereProperties {
    /// Constructs an atmosphere where density smoothly decays to 0 at `atmosphere_height`.
    /// `cutoff_fraction` is the relative density before fade-out (default ~1e-4 / 0.01%).
    pub fn new(surface_density: f32, atmosphere_height: f32) -> Self {
        // Target 0.01% remaining density at the boundary before zeroing
        const CUTOFF_FRACTION: f32 = 1e-4;
        let scale_height = atmosphere_height / CUTOFF_FRACTION.ln().abs();

        Self {
            sea_level_density: surface_density,
            scale_height,
            atmosphere_height,
        }
    }

    #[inline]
    pub fn density_at_altitude(&self, altitude: f32) -> f32 {
        if altitude <= 0.0 {
            return self.sea_level_density;
        }
        if altitude >= self.atmosphere_height {
            return 0.0;
        }

        // Shifted exponential: reaches exactly 0.0 at atmosphere_height
        let exp_current = (-altitude / self.scale_height).exp();
        let exp_top = (-self.atmosphere_height / self.scale_height).exp();

        self.sea_level_density * (exp_current - exp_top) / (1.0 - exp_top)
    }
}

pub fn create_atmosphere(
    body_radius: f32,
    height: f32,
    sea_level_density: f32,
    ground_albedo: Vec3,
    scattering_medium: Handle<ScatteringMedium>,
) -> (Atmosphere, AtmosphereProperties) {
    let rendering_atmosphere = Atmosphere {
        inner_radius: body_radius,
        outer_radius: body_radius + height,
        ground_albedo,
        medium: scattering_medium,
    };
    let properties = AtmosphereProperties::new(sea_level_density, height);

    (rendering_atmosphere, properties)
}

#[derive(Component, Default, Copy, Clone, Debug)]
#[require(ClosestBody)]
pub struct LocalAirDensity(pub f32);

pub fn calculate_local_air_density(
    grid: Single<&Grid, With<BigSpace>>,
    mut locations: Query<(&mut LocalAirDensity, &ClosestBody, CellTransformReadOnly)>,
    bodies: Query<(
        &CelestialBody,
        Option<&AtmosphereProperties>,
        CellTransformReadOnly,
    )>,
) {
    for (mut target, maybe_body, ship_transform) in locations.iter_mut() {
        // Default to vacuum
        target.0 = 0.0;

        let Some(body_entity) = maybe_body.0 else {
            continue;
        };
        let Ok((body, Some(atmosphere), body_transform)) = bodies.get(body_entity) else {
            continue;
        };

        let ship_pos = grid.grid_position_double(ship_transform.cell, ship_transform.transform);
        let body_pos = grid.grid_position_double(body_transform.cell, body_transform.transform);

        let altitude = ship_pos.distance(body_pos) as f32 - body.radius;
        target.0 = atmosphere.density_at_altitude(altitude);
    }
}
