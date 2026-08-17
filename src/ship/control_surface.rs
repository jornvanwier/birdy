use crate::input::Action;
use crate::ship::Ship;
use avian3d::prelude::AngularVelocity;

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
    ships: Query<
        (
            &ActionState<Action>,
            &AngularVelocity,
            &Transform,
            &Children,
        ),
        With<Ship>,
    >,
    mut actuators: Query<(&ControlSurfaceOrientation, &mut ControlSurfaceActuator)>,
) {
    // Damping gains (rate gyro feedback)
    const PITCH_RATE_GAIN: f32 = 0.50;
    const ROLL_RATE_GAIN: f32 = 0.20;
    const YAW_RATE_GAIN: f32 = 0.60;

    for (action_state, ang_vel, transform, children) in &ships {
        let Vec2 {
            x: roll_in,
            y: pitch_in,
        } = action_state.clamped_axis_pair(&Action::RollPitch);
        let yaw_in = action_state.value(&Action::Yaw);

        // Convert world-space angular velocity into ship local space
        let local_ang_vel = transform.rotation.inverse() * ang_vel.0;

        // SAS adds an opposing deflection proportional to rotation speed in local frame
        let pitch_cmd = pitch_in + local_ang_vel.x * PITCH_RATE_GAIN;
        let roll_cmd = roll_in + local_ang_vel.z * ROLL_RATE_GAIN;
        let yaw_cmd = yaw_in + local_ang_vel.y * YAW_RATE_GAIN;

        for child in children.iter() {
            if let Ok((orientation, mut actuator)) = actuators.get_mut(child) {
                actuator.target_deflection = match *orientation {
                    ControlSurfaceOrientation::Pitch => pitch_cmd.clamp(-1.0, 1.0),
                    ControlSurfaceOrientation::Roll { negate } => {
                        (roll_cmd * if negate { -1. } else { 1. }).clamp(-1.0, 1.0)
                    }
                    ControlSurfaceOrientation::Yaw => yaw_cmd.clamp(-1.0, 1.0),
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
