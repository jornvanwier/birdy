use avian3d::prelude::{Forces, Mass, WriteRigidBodyForces};
use bevy::prelude::*;

/// Directional gravitational acceleration (m/s^2) acting on the entity.
#[derive(Component, Default, Copy, Clone, Debug, Deref, DerefMut)]
pub struct LocalGravity(pub Vec3);

impl LocalGravity {
    /// Gravitational acceleration magnitude in m/s^2.
    #[inline]
    pub fn magnitude(&self) -> f32 {
        self.0.length()
    }

    /// Acceleration in Earth Gs (1 G ≈ 9.80665 m/s^2) for cockpit HUD displays.
    #[inline]
    pub fn g_force(&self) -> f32 {
        self.magnitude() / 9.80665
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