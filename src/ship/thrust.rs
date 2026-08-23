use crate::input::Action;
use crate::ship::sensors::ThrustMeasurement;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Component, Clone, Default, Reflect)]
pub struct Thrust {
    pub current_throttle: f32,
    pub target_throttle: f32,
    pub peak_thrust: f32,
    change_rate: f32,
    smoothness: f32,
}

impl Thrust {
    pub fn new(initial_value: f32, peak_thrust: f32, change_rate: f32, smoothness: f32) -> Self {
        Self {
            current_throttle: initial_value,
            target_throttle: initial_value,
            peak_thrust,
            change_rate,
            smoothness,
        }
    }

    pub fn adjust_target_delta(&mut self, delta: f32, delta_secs: f32) {
        if delta != 0.0 {
            self.target_throttle += delta * self.change_rate * delta_secs;
            self.target_throttle = self.target_throttle.clamp(0.0, 1.0);
        }
    }

    pub fn update(&mut self, delta_secs: f32) {
        self.current_throttle += (self.target_throttle - self.current_throttle)
            * (self.smoothness * delta_secs).min(1.0);
    }
}

pub(crate) fn handle_throttle(
    time: Res<Time>,
    action_query: Query<&ActionState<Action>>,
    mut thrust_query: Query<(&ChildOf, &mut Thrust)>,
) {
    let delta_t = time.delta_secs();

    for (child_of, mut throttle) in thrust_query.iter_mut() {
        if let Ok(action_state) = action_query.get(child_of.parent()) {
            if action_state.pressed(&Action::FullThrottle) {
                throttle.target_throttle = 1.;
            } else if action_state.pressed(&Action::CutThrottle) {
                throttle.target_throttle = 0.;
            } else {
                let throttle_dir = action_state.value(&Action::Throttle);
                throttle.adjust_target_delta(throttle_dir, delta_t);
            }
        }

        throttle.update(delta_t);
    }
}

pub fn apply_thrust(
    mut ships: Query<(
        Forces,
        &Transform,
        &Children,
        Option<&mut ThrustMeasurement>,
    )>,
    thrusters: Query<(&Thrust, &Transform)>,
) {
    for (mut forces, ship_transform, children, thrust_diagnostic) in ships.iter_mut() {
        let ship_rot = ship_transform.rotation;

        let mut total_thrust = Vec3::ZERO;
        for child in children.iter() {
            let Ok((thrust, thruster_transform)) = thrusters.get(child) else {
                continue;
            };

            let magnitude = thrust.current_throttle * thrust.peak_thrust;
            if magnitude <= 0.0 {
                continue;
            }

            // Direction the thruster points in world space (thrust pushes along ship's forward/-Z)
            let thruster_world_rot = ship_rot * thruster_transform.rotation;
            let thrust_direction = thruster_world_rot * Vec3::NEG_Z;
            let thrust_force = thrust_direction * magnitude;

            // Lever arm from ship center of mass to thruster in world coordinates
            let arm = ship_rot * thruster_transform.translation;

            forces.apply_force(thrust_force);
            forces.apply_torque(arm.cross(thrust_force));

            total_thrust += thrust_force;
        }

        if let Some(mut thrust_measurement) = thrust_diagnostic {
            *thrust_measurement = ThrustMeasurement(total_thrust);
        }
    }
}
