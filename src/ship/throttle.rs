use crate::input::Action;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Component, Clone, Default, Reflect)]
pub struct Throttle {
    pub current: f32,
    pub target: f32,
    change_rate: f32,
    smoothness: f32,
}

impl Throttle {
    pub fn new(initial_value: f32, change_rate: f32, smoothness: f32) -> Self {
        Self {
            current: initial_value,
            target: initial_value,
            change_rate,
            smoothness,
        }
    }

    pub fn adjust_target_delta(&mut self, delta: f32, delta_secs: f32) {
        if delta != 0.0 {
            self.target += delta * self.change_rate * delta_secs;
            self.target = self.target.clamp(0.0, 1.0);
        }
    }

    pub fn update(&mut self, delta_secs: f32) {
        self.current += (self.target - self.current) * (self.smoothness * delta_secs).min(1.0);
    }
}
pub(crate) fn handle_throttle(
    time: Res<Time>,
    mut query: Query<(&ActionState<Action>, &mut Throttle)>,
) {
    for (action_state, mut throttle) in query.iter_mut() {
        let delta_t = time.delta_secs();
        if action_state.pressed(&Action::FullThrottle) {
            throttle.target = 1.;
        } else if action_state.pressed(&Action::CutThrottle) {
            throttle.target = 0.;
        } else {
            let throttle_dir = action_state.value(&Action::Throttle);
            throttle.adjust_target_delta(throttle_dir, delta_t);
        }

        throttle.update(delta_t);
    }
}
