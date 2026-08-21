use bevy::prelude::{Commands, Component, GlobalTransform, Query, Reflect, Res, Time};
use leafwing_input_manager::action_state::ActionState;
use crate::audio::FireGunEvent;
use crate::input::Action;

/// Rotary gun physics & spool state
#[derive(Component, Clone, Reflect)]
pub struct RotaryGun {
    /// 0.0 (stationary) to 1.0 (max RPM)
    pub current_spool: f32,
    /// Acceleration rate of the drive motor (~0.7s to reach full cyclic speed)
    pub spool_up_rate: f32,
    /// Deceleration rate of the spinning barrels when trigger released (~1.1s to halt)
    pub spool_down_rate: f32,
    /// Spool threshold before the feed mechanism begins chambering rounds
    pub min_fire_spool: f32,
    /// Maximum cyclic rate (shots/sec at 100% RPM, e.g. 36.0 = ~2160 RPM)
    pub max_fire_rate: f32,
    /// Time accumulator for sub-frame accurate shot timing
    pub fire_accumulator: f32,
}

impl Default for RotaryGun {
    fn default() -> Self {
        Self {
            current_spool: 0.0,
            spool_up_rate: 1.4,   // Takes ~0.7s to reach max RPM
            spool_down_rate: 0.9, // Takes ~1.1s to spin down
            min_fire_spool: 0.32, // Spools for ~0.23s before first shot
            max_fire_rate: 36.0,  // 36 rounds/sec at peak speed
            fire_accumulator: 0.0,
        }
    }
}

/// Spools the gun motor and dispatches FireGunEvents with acceleration curve
pub fn handle_gun_firing(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&ActionState<Action>, &GlobalTransform, &mut RotaryGun)>,
) {
    let dt = time.delta_secs();

    for (action_state, global_transform, mut gun) in &mut query {
        let is_firing = action_state.pressed(&Action::Fire);

        // 1. Spool acceleration & deceleration
        if is_firing {
            gun.current_spool = (gun.current_spool + gun.spool_up_rate * dt).min(1.0);
        } else {
            gun.current_spool = (gun.current_spool - gun.spool_down_rate * dt).max(0.0);
        }

        // 2. Dynamic rate firing: starts at ~11 rounds/sec and ramps up to 36 rounds/sec
        if is_firing && gun.current_spool >= gun.min_fire_spool {
            let current_rate = gun.current_spool * gun.max_fire_rate;
            let shot_interval = 1.0 / current_rate;

            gun.fire_accumulator += dt;
            while gun.fire_accumulator >= shot_interval {
                gun.fire_accumulator -= shot_interval;

                commands.trigger(FireGunEvent {
                    transform: global_transform.compute_transform(),
                });
            }
        } else {
            // Reset accumulator when below firing threshold
            gun.fire_accumulator = 0.0;
        }
    }
}