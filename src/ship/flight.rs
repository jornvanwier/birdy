use crate::input::Action;
use crate::ship::throttle::Throttle;
use avian3d::prelude::forces::ForcesItem;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FlightModel {
    pub max_thrust: f32,         // Engine thrust in Newtons (N)
    pub wing_area: f32,          // Wing surface area (m^2)
    pub lift_coefficient: f32,   // Base lift coefficient (C_L)
    pub lift_slope: f32,         // Rate of C_L gain per radian of AoA (C_L_alpha)
    pub drag_coefficient: f32,   // Base drag coefficient (C_D)
    pub induced_drag_coeff: f32, // Extra drag when unaligned with velocity

    /// Center of Lift offset in local space relative to Center of Mass (+Z is behind CoM)
    pub center_of_lift_offset: Vec3,
    /// Center of Thrust offset in local space relative to Center of Mass (+Z is behind CoM)
    pub center_of_thrust_offset: Vec3,

    /// Tail fin location relative to Center of Mass (+Z is behind CoM)
    pub tail_offset: Vec3,

    /// Surface area of the tail fin (provides passive weathercocking stability)
    pub tail_fin_area: f32,

    pub aileron_wing_position: f32,

    /// Control surface force multipliers
    pub elevator_authority: f32,
    pub rudder_authority: f32,
    pub aileron_authority: f32,

    /// Rotational damping factors in local space (prevents endless spin/oscillations)
    pub angular_damping: Vec3,
}

#[derive(Component, Reflect, Default)]
pub struct FlightTelemetry {
    pub velocity: Vec3,
    pub thrust: Vec3,
    pub drag: Vec3,
    pub lift: Vec3,
    pub angle_of_attack: f32,
    pub slip_ratio: f32,
    pub dynamic_pressure: f32,
}

impl Default for FlightModel {
    fn default() -> Self {
        Self {
            max_thrust: 35000.0,
            wing_area: 12.0,
            lift_coefficient: 0.1,
            lift_slope: 5.7,
            drag_coefficient: 0.05,
            induced_drag_coeff: 0.2,
            center_of_lift_offset: Vec3::ZERO,
            center_of_thrust_offset: Vec3::new(0.0, 0.0, 1.5),
            // 1.2m behind CoM
            tail_offset: Vec3::new(0.0, 0.0, 1.2),
            tail_fin_area: 0.5,
            aileron_wing_position: 1.0,
            elevator_authority: 0.01,
            rudder_authority: 0.05,
            aileron_authority: 0.01,
            angular_damping: Vec3::new(1.2, 8.0, 2.0),
        }
    }
}

fn apply_force_at_offset(forces: &mut ForcesItem, force: Vec3, local_offset: Vec3, rotation: Quat) {
    forces.apply_force(force);
    let world_offset = rotation * local_offset;
    forces.apply_torque(world_offset.cross(force));
}

pub fn apply_flight_forces(
    mut query: Query<(
        Forces,
        Mut<FlightTelemetry>,
        &Transform,
        &Throttle,
        &FlightModel,
        &ActionState<Action>,
    )>,
) {
    let air_density = 1.225;

    for (mut forces, mut telemetry, transform, throttle, flight_model, action_state) in
        query.iter_mut()
    {
        let up = *transform.up();
        let right = *transform.right();
        let forward = *transform.forward();
        let rotation = transform.rotation;

        let velocity = forces.linear_velocity();
        let speed = velocity.length();

        let world_angular_velocity = forces.angular_velocity();
        let local_angular_velocity = rotation.inverse() * world_angular_velocity;

        // Thrust
        let thrust_magnitude = flight_model.max_thrust * throttle.current;
        let thrust_force = forward * thrust_magnitude;
        apply_force_at_offset(
            &mut forces,
            thrust_force,
            flight_model.center_of_thrust_offset,
            rotation,
        );

        if speed > 0.01 {
            let vel_dir = velocity / speed;
            let dynamic_pressure = calculate_dynamic_pressure(air_density, speed);

            let alignment = forward.dot(vel_dir).abs();

            // Drag
            let total_drag_coeff =
                flight_model.drag_coefficient + (1.0 - alignment) * flight_model.induced_drag_coeff;
            let drag_magnitude = dynamic_pressure * flight_model.wing_area * total_drag_coeff;
            let drag_force = -vel_dir * drag_magnitude;
            forces.apply_force(drag_force);

            // Lift
            let lift_dir = (up - vel_dir * up.dot(vel_dir)).normalize_or_zero();
            let local_vel = rotation.inverse() * vel_dir;
            // Cap angle of attack at ~25 degrees (0.43 rad) to simulate wing stall
            let angle_of_attack = (-local_vel.y).atan2(-local_vel.z).clamp(-0.43, 0.43);
            let effective_cl =
                flight_model.lift_coefficient + (angle_of_attack * flight_model.lift_slope);

            let lift_magnitude = dynamic_pressure * flight_model.wing_area * effective_cl;
            let lift_force = lift_dir * lift_magnitude;
            apply_force_at_offset(
                &mut forces,
                lift_force,
                flight_model.center_of_lift_offset,
                rotation,
            );

            // Tail fin side drag (dynamic weathercocking)
            // Combined linear side speed + rotational speed of the tail fin (omega * radius)
            let tail_rotational_side_speed = local_angular_velocity.y * flight_model.tail_offset.z;
            let total_tail_side_speed = velocity.dot(right) + tail_rotational_side_speed;
            let slip_ratio = total_tail_side_speed / speed;

            let side_drag_force =
                -right * (dynamic_pressure * flight_model.tail_fin_area * slip_ratio);
            apply_force_at_offset(
                &mut forces,
                side_drag_force,
                flight_model.tail_offset,
                rotation,
            );

            // Input handling
            // TODO trim
            let Vec2 {
                x: roll_input,
                y: pitch_input,
            } = action_state.clamped_axis_pair(&Action::RollPitch);
            let yaw_input = action_state.value(&Action::Yaw);

            // Elevator
            let elevator_force =
                up * (pitch_input * flight_model.elevator_authority * dynamic_pressure);
            apply_force_at_offset(
                &mut forces,
                elevator_force,
                flight_model.tail_offset,
                rotation,
            );

            // Rudder
            let rudder_force =
                right * (-yaw_input * flight_model.rudder_authority * dynamic_pressure);
            apply_force_at_offset(
                &mut forces,
                rudder_force,
                flight_model.tail_offset,
                rotation,
            );

            // Aileron Differential
            let left_wing_offset = Vec3::new(-flight_model.aileron_wing_position, 0.0, 0.0);
            let right_wing_offset = -left_wing_offset;

            let aileron_force =
                up * (roll_input * flight_model.aileron_authority * dynamic_pressure);
            apply_force_at_offset(&mut forces, aileron_force, left_wing_offset, rotation);
            apply_force_at_offset(&mut forces, -aileron_force, right_wing_offset, rotation);

            // Angular Damping (Prevents harmonic spring bobbing)
            let local_damping = -local_angular_velocity * flight_model.angular_damping;
            let world_damping = rotation * local_damping;
            forces.apply_angular_acceleration(world_damping);

            *telemetry = FlightTelemetry {
                velocity,
                thrust: thrust_force,
                drag: drag_force,
                lift: lift_force,
                angle_of_attack,
                slip_ratio,
                dynamic_pressure,
            }
        }
    }
}

fn calculate_dynamic_pressure(air_density: f32, speed: f32) -> f32 {
    0.5 * air_density * speed.powi(2)
}
