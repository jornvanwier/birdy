use avian3d::prelude::{Forces, Mass, WriteRigidBodyForces};
use bevy::prelude::*;

/// Directional gravitational acceleration (m/s^2) acting on the entity.
#[derive(Component, Default, Copy, Clone, Debug, Deref, DerefMut)]
pub struct LocalGravity(pub Vec3);

pub const STANDARD_G: f32 = 9.80665;

impl LocalGravity {
    /// Gravitational acceleration magnitude in m/s^2.
    #[inline]
    pub fn magnitude(&self) -> f32 {
        self.0.length()
    }
}

pub fn apply_local_gravity(
    mut query: Query<(Forces, &LocalGravity, &Mass)>,
) {
    for (mut force, gravity, mass) in &mut query {
        if gravity.0 != Vec3::ZERO {
            // F = m * a
            force.apply_force(gravity.0 * mass.0);
        }
    }
}