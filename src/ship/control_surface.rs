use crate::ship::Ship;

use crate::ship::fcs::FlightControlCommand;
use bevy::prelude::*;

#[derive(Component, Reflect, Eq, PartialEq, Copy, Clone, Default)]
pub enum ControlSurfacePosition {
    #[default]
    Left,
    Right,
}

#[derive(Component, Reflect, Clone, Default)]
pub enum ControlSurfaceOrientation {
    #[default]
    Pitch,
    Roll(ControlSurfacePosition),
    RollPitch(ControlSurfacePosition),
    Yaw,
}

pub fn set_control_surface_targets(
    ships: Query<(&FlightControlCommand, &Children), With<Ship>>,
    mut actuators: Query<(&ControlSurfaceOrientation, &mut ControlSurfaceActuator)>,
) {
    for (command, children) in &ships {
        let FlightControlCommand {
            pitch: pitch_cmd,
            roll: roll_cmd,
            yaw: yaw_cmd,
            ..
        } = command;

        for child in children.iter() {
            if let Ok((orientation, mut actuator)) = actuators.get_mut(child) {
                actuator.target_deflection = match *orientation {
                    ControlSurfaceOrientation::Pitch => *pitch_cmd,
                    ControlSurfaceOrientation::Roll(side) => {
                        *roll_cmd
                            * match side {
                                ControlSurfacePosition::Left => 1.,
                                ControlSurfacePosition::Right => -1.,
                            }
                    }
                    ControlSurfaceOrientation::RollPitch(side) => match side {
                        ControlSurfacePosition::Left => *pitch_cmd + *roll_cmd,
                        ControlSurfacePosition::Right => *pitch_cmd - *roll_cmd,
                    },
                    ControlSurfaceOrientation::Yaw => *yaw_cmd,
                }
                .clamp(-1.0, 1.0);
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
