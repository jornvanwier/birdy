use crate::input::Action;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Component, Reflect)]
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

    pub fn update(&mut self, delta: f32, delta_secs: f32) {
        if delta != 0.0 {
            self.target += delta * self.change_rate * delta_secs;
            self.target = self.target.clamp(0.0, 1.0);
        }

        self.current += (self.target - self.current) * (self.smoothness * delta_secs).min(1.0);
    }
}
pub(crate) fn handle_throttle(
    time: Res<Time>,
    mut query: Query<(&ActionState<Action>, &mut Throttle)>,
) {
    info!("Handling throttle");

    for (action_state, mut throttle) in query.iter_mut() {
        let delta = action_state.value(&Action::Throttle);

        throttle.update(delta, time.delta_secs());
    }
}
