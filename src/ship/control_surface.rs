use crate::input::Action;
use crate::ship::Ship;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Component, Reflect, Clone, Default)]
pub enum ControlSurfaceOrientation {
    #[default]
    Pitch,
    Roll {
        negate: bool,
    },
    Yaw,
}

pub fn set_control_surface_targets(
    ships: Query<(&ActionState<Action>, &Children), With<Ship>>,
    mut actuators: Query<(&ControlSurfaceOrientation, &mut ControlSurfaceActuator)>,
) {
    for (action_state, children) in &ships {
        let Vec2 {
            x: roll_input,
            y: pitch_input,
        } = action_state.clamped_axis_pair(&Action::RollPitch);
        let yaw_input = action_state.value(&Action::Yaw);

        for child in children.iter() {
            if let Ok((orientation, mut actuator)) = actuators.get_mut(child) {
                actuator.target_deflection = match *orientation {
                    ControlSurfaceOrientation::Pitch => pitch_input,
                    ControlSurfaceOrientation::Roll { negate } => {
                        roll_input * if negate { -1. } else { 1. }
                    }
                    ControlSurfaceOrientation::Yaw => yaw_input,
                };
            }
        }
    }
}

#[derive(Component, Reflect, Copy, Clone)]
pub struct ControlSurfaceActuator {
    pub max_angle: f32,         // Max deflection in radians
    pub speed: f32,             // Radians per second
    pub target_deflection: f32, // -1.0 to 1.0 from input
    pub base_rotation: Quat,    // Resting orientation (e.g. 90 deg Z for vertical tail)
}

impl Default for ControlSurfaceActuator {
    fn default() -> Self {
        Self {
            max_angle: f32::to_radians(25.0),
            speed: 5.0,
            target_deflection: 0.0,
            base_rotation: Quat::IDENTITY,
        }
    }
}

pub fn update_control_surfaces(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &ControlSurfaceActuator)>,
) {
    for (mut transform, actuator) in query.iter_mut() {
        let target_angle = actuator.target_deflection * actuator.max_angle;
        let deflection_rot = Quat::from_rotation_x(target_angle);
        let target_rot = actuator.base_rotation * deflection_rot;

        transform.rotation = transform
            .rotation
            .rotate_towards(target_rot, actuator.speed * time.delta_secs());
    }
}
