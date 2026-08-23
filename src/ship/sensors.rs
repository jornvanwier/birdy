use crate::ship::air_density::calculate_scalar_dynamic_pressure;
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
    /// Altitude above sea level in meters
    pub altitude: f32,
    /// Vertical climb/descent rate in m/s
    pub vertical_speed: f32,

    // Internal cache for calculating acceleration / G-force across fixed steps
    #[reflect(ignore)]
    pub(crate) prev_linear_velocity: Vec3,
}

pub fn update_flight_sensors(
    time: Res<Time>,
    mut query: Query<(&Transform, &LinearVelocity, &AngularVelocity, &mut FlightSensorData)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let air_density = 1.225; // kg/m^3 (or sample from atmosphere altitude)
    let gravity = Vec3::new(0.0, -9.81, 0.0);

    for (transform, lin_vel, ang_vel, mut air_data) in query.iter_mut() {
        let rot = transform.rotation;
        let v_world = lin_vel.0;
        let speed = v_world.length();

        // Transform velocities into aircraft body frame (-Z Forward, Y Up, X Right)
        let v_body = rot.inverse() * v_world;
        let omega_body = rot.inverse() * ang_vel.0;

        // Kinematic angles
        // AoA = pitch plane angle between forward (-Z) and up (Y)
        let aoa = (-v_body.y).atan2(-v_body.z);
        // Sideslip = yaw plane angle between forward (-Z) and right (X)
        let beta = v_body.x.atan2(-v_body.z);

        // Dynamic pressure
        let q = calculate_scalar_dynamic_pressure(air_density, speed);

        // Calculate accelerometer-felt G-Force: (a_inertial - g) / 9.81
        let a_world = (v_world - air_data.prev_linear_velocity) / dt;
        let felt_accel_world = a_world - gravity;
        let felt_accel_local = rot.inverse() * felt_accel_world;
        let g_force_local = felt_accel_local / gravity.y;

        air_data.prev_linear_velocity = v_world;
        air_data.true_airspeed = speed;
        air_data.dynamic_pressure = q;
        air_data.aoa = if speed > 1.0 { aoa } else { 0.0 };
        air_data.sideslip = if speed > 1.0 { beta } else { 0.0 };
        air_data.local_ang_vel = omega_body;
        air_data.g_force_local = g_force_local;

        // TODO calculate wrt to current body
        air_data.altitude = transform.translation.y;
        air_data.vertical_speed = v_world.y;
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
