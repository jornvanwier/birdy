use crate::ship::Ship;

use crate::ship::fcs::FlightControlCommand;
use bevy::prelude::*;

// src/ship/control_surface.rs

#[derive(Component, Reflect, Clone, Copy, Debug, Default)]
pub struct ControlMix {
    pub pitch: f32, // -1.0 to 1.0 (authority & polarity)
    pub roll: f32,
    pub yaw: f32,
    pub flaps: f32,
}

#[allow(unused)]
impl ControlMix {
    pub const PITCH: Self = Self { pitch: 1.0, roll: 0.0, yaw: 0.0, flaps: 0.0 };
    pub const YAW: Self = Self { pitch: 0.0, roll: 0.0, yaw: 1.0, flaps: 0.0 };

    /// Standard ailerons (pure roll differential)
    pub fn aileron(side_sign: f32) -> Self {
        Self { roll: side_sign, ..default() }
    }

    /// Flaperon (roll + symmetric flap droop)
    pub fn flaperon(side_sign: f32, flap_weight: f32) -> Self {
        Self {
            roll: side_sign * (1.0 - flap_weight),
            flaps: flap_weight,
            ..default()
        }
    }

    /// Tailerons / Elevons (pitch + roll differential)
    pub fn taileron(side_sign: f32) -> Self {
        Self {
            pitch: 1.0,
            roll: side_sign * 0.5,
            ..default()
        }
    }

    /// Ruddervator (V-Tail / Canted surfaces: pitch + yaw)
    pub fn ruddervator(side_sign: f32) -> Self {
        Self {
            pitch: 1.0,
            yaw: side_sign,
            ..default()
        }
    }
}

pub fn set_control_surface_targets(
    ships: Query<(&FlightControlCommand, &Children), With<Ship>>,
    mut actuators: Query<(&ControlMix, &mut ControlSurfaceActuator)>,
) {
    for (cmd, children) in &ships {
        for child in children.iter() {
            if let Ok((mix, mut actuator)) = actuators.get_mut(child) {
                // Vector dot product of commands and surface authority weights
                let total_target = (cmd.pitch * mix.pitch)
                    + (cmd.roll * mix.roll)
                    + (cmd.yaw * mix.yaw)
                    + (cmd.flaps * mix.flaps);

                actuator.target_deflection = total_target.clamp(-1.0, 1.0);
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
