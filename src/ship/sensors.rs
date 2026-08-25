use crate::environment::air_density::calculate_scalar_dynamic_pressure;
use avian3d::prelude::*;
use bevy::prelude::*;
use std::ops::AddAssign;

/// Primary flight sensors / air data computer.
/// Calculated at the start of the frame from rigid body state.
#[derive(Component, Reflect, Clone, Default, Debug)]
pub struct FlightSensorData {
    /// True airspeed in m/s
    pub true_airspeed: f32,
    /// Dynamic pressure (q = 0.5 * rho * v^2) in Pa
    pub dynamic_pressure: f32,
    /// Angle of attack in radians (body reference line)
    pub aoa: f32,
    /// Sideslip angle (beta) in radians
    pub sideslip: f32,
    /// Felt G-force (load factor) in body axes (X=Right, Y=Up, Z=Back)
    /// Y-axis is vertical Gs (1.0 = level unaccelerated flight)
    pub g_force_local: Vec3,
    /// Angular velocity in body local axes (Pitch, Roll, Yaw rates)
    pub local_ang_vel: Vec3,
    /// Vertical climb/descent rate in m/s
    pub vertical_speed: f32,

    // Internal cache for calculating acceleration / G-force across fixed steps
    #[reflect(ignore)]
    pub(crate) prev_linear_velocity: Vec3,
}

use crate::environment::{LocalAirDensity, LocalGravity, PlanetaryFrame, gravity};

pub fn update_flight_sensors(
    time: Res<Time>,
    mut query: Query<(
        &Transform,
        &LinearVelocity,
        &AngularVelocity,
        &LocalAirDensity,
        &LocalGravity,
        &PlanetaryFrame,
        &mut FlightSensorData,
    )>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (transform, lin_vel, ang_vel, air_density, local_gravity, frame, mut air_data) in
        query.iter_mut()
    {
        let rot = transform.rotation;
        let v_world = lin_vel.0;
        let speed = v_world.length();

        let v_body = rot.inverse() * v_world;
        let omega_body = rot.inverse() * ang_vel.0;

        let aoa = (-v_body.y).atan2(-v_body.z);
        let beta = v_body.x.atan2(-v_body.z);
        let q = calculate_scalar_dynamic_pressure(air_density.0, speed);

        // Accelerometer load factor (G-force)
        let a_world = (v_world - air_data.prev_linear_velocity) / dt;
        let felt_accel_world = a_world - local_gravity.0;
        let felt_accel_local = rot.inverse() * felt_accel_world;
        let g_force_local = felt_accel_local / gravity::STANDARD_G;

        // Local Up from surface rotation (+Y in local ground frame)
        let local_up = frame.rotation * Vec3::Y;

        air_data.prev_linear_velocity = v_world;
        air_data.true_airspeed = speed;
        air_data.dynamic_pressure = q;
        air_data.aoa = if speed > 1.0 { aoa } else { 0.0 };
        air_data.sideslip = if speed > 1.0 { beta } else { 0.0 };
        air_data.local_ang_vel = omega_body;
        air_data.g_force_local = g_force_local;
        air_data.vertical_speed = v_world.dot(local_up);
    }
}

#[derive(Component, Reflect, Clone, Default, Debug)]
pub struct LiftAndDragMeasurement {
    pub lift: Vec3,
    pub drag: Vec3,
}

impl AddAssign for LiftAndDragMeasurement {
    fn add_assign(&mut self, rhs: Self) {
        self.lift = self.lift + rhs.lift;
        self.drag = self.drag + rhs.drag;
    }
}

#[derive(Component, Reflect, Clone, Default, Debug)]
pub struct ThrustMeasurement(pub Vec3);
