use avian3d::parry::glamx::Vec3;

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