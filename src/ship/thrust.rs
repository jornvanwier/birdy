use crate::input::Action;
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
    mut query: Query<(&ActionState<Action>, &mut Thrust)>,
) {
    for (action_state, mut throttle) in query.iter_mut() {
        let delta_t = time.delta_secs();
        if action_state.pressed(&Action::FullThrottle) {
            throttle.target_throttle = 1.;
        } else if action_state.pressed(&Action::CutThrottle) {
            throttle.target_throttle = 0.;
        } else {
            let throttle_dir = action_state.value(&Action::Throttle);
            throttle.adjust_target_delta(throttle_dir, delta_t);
        }

        throttle.update(delta_t);
    }
}

pub fn apply_thrust(mut query: Query<(Forces, &Transform, &Thrust)>) {
    for (mut forces, transform, thrust) in query.iter_mut() {
        let magnitude = thrust.current_throttle * thrust.peak_thrust;
        let direction = transform.forward();

        forces.apply_force(direction * magnitude);
    }
}
