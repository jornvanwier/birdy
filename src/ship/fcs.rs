use crate::input::Action;
use crate::ship::Player;
use crate::ship::sensors::FlightSensorData;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

/// Represents normalized pilot/autopilot control intent (-1.0 to 1.0)
#[derive(Component, Reflect, Clone, Copy, Debug, Default)]
pub struct FlightControlCommand {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
    pub throttle: f32,
}

#[derive(Component, Reflect, Clone)]
pub struct AttitudeHoldState {
    pub target_rotation: Option<Quat>,
    pub stick_deadzone: f32,

    // --- Military FBW Assists ---
    /// Automatically applies top-rudder at high bank angles to support knife-edge flight
    pub enable_top_rudder_assist: bool,
    pub top_rudder_gain: f32, // e.g. 0.4 - 0.7

    /// Automatically applies back-pressure in turns to prevent altitude loss
    pub enable_turn_pitch_trim: bool,
    pub turn_pitch_trim_gain: f32, // e.g. 0.15 - 0.3
}

impl Default for AttitudeHoldState {
    fn default() -> Self {
        Self {
            target_rotation: None,
            stick_deadzone: 0.05,
            enable_top_rudder_assist: true,
            top_rudder_gain: 0.5,
            enable_turn_pitch_trim: true,
            turn_pitch_trim_gain: 0.2,
        }
    }
}

pub fn process_player_flight_inputs(
    mut query: Query<(&ActionState<Action>, &mut FlightControlCommand), With<Player>>,
) {
    for (action_state, mut cmd) in &mut query {
        let Vec2 { x: roll, y: pitch } = action_state.clamped_axis_pair(&Action::RollPitch);
        let yaw = action_state.value(&Action::Yaw);
        let throttle = action_state.value(&Action::Throttle);

        *cmd = FlightControlCommand {
            pitch,
            roll,
            yaw,
            throttle,
        };
    }
}

pub fn apply_attitude_hold_assist(
    mut query: Query<(
        &Transform,
        &FlightSensorData,
        &mut AttitudeHoldState,
        &mut FlightControlCommand,
    )>,
) {
    // Controller gains
    const P_PITCH: f32 = 6.0;
    const D_PITCH: f32 = 0.8;

    const P_ROLL: f32 = 5.0;
    const D_ROLL: f32 = 0.5;

    const D_YAW: f32 = 0.6;

    // TODO Add attitude hold disable, add back angular damping in that case
    //     cmd.pitch = (cmd.pitch + sensors.local_ang_vel.x * D_PITCH).clamp(-1.0, 1.0);
    //     cmd.roll = (cmd.roll + sensors.local_ang_vel.z * D_ROLL).clamp(-1.0, 1.0);
    //     cmd.yaw = (cmd.yaw + sensors.local_ang_vel.y * D_YAW).clamp(-1.0, 1.0);

    for (transform, sensors, mut hold_state, mut cmd) in &mut query {
        let deadzone = hold_state.stick_deadzone;

        let pitch_active = cmd.pitch.abs() > deadzone;
        let roll_active = cmd.roll.abs() > deadzone;
        let yaw_input = if cmd.yaw.abs() > deadzone { cmd.yaw } else { 0.0 };

        // -----------------------------------------------------------------
        // 1. GRAVITY VECTOR DECOMPOSITION (Military FBW Kinematics)
        // -----------------------------------------------------------------
        // Transform world UP (0, 1, 0) into local aircraft space:
        // - local_up.y = wing levelness (1.0 = level, 0.0 = 90 deg bank)
        // - local_up.x = lateral tilt (negative = right bank, positive = left bank)
        let local_up = transform.rotation.conjugate() * Vec3::Y;

        // Auto Turn Pitch Compensation: 1/cos(bank) - 1
        let auto_turn_pitch = if hold_state.enable_turn_pitch_trim {
            let cos_bank = local_up.y.clamp(0.35, 1.0); // clamp to prevent infinite pitch at 90 deg
            (1.0 / cos_bank - 1.0) * hold_state.turn_pitch_trim_gain
        } else {
            0.0
        };

        // Auto Top-Rudder Assist: proportional to lateral gravity projection on the wings
        let auto_top_rudder = if hold_state.enable_top_rudder_assist {
            local_up.x * hold_state.top_rudder_gain
        } else {
            0.0
        };

        // -----------------------------------------------------------------
        // 2. PITCH & ROLL CHANNELS
        // -----------------------------------------------------------------
        if pitch_active || roll_active {
            // PILOT IS ACTIVELY FLYING:
            // Disengage attitude target so it captures fresh on stick release
            hold_state.target_rotation = None;

            // Stability Augmentation System (SAS):
            // Direct input + Gyro Rate Damping + Auto-Turn Pitch Bias
            if pitch_active {
                cmd.pitch = (cmd.pitch + auto_turn_pitch + sensors.local_ang_vel.x * D_PITCH).clamp(-1.0, 1.0);
            } else {
                cmd.pitch = (auto_turn_pitch + sensors.local_ang_vel.x * D_PITCH).clamp(-1.0, 1.0);
            }

            if roll_active {
                cmd.roll = (cmd.roll + sensors.local_ang_vel.z * D_ROLL).clamp(-1.0, 1.0);
            } else {
                cmd.roll = (sensors.local_ang_vel.z * D_ROLL).clamp(-1.0, 1.0);
            }
        } else {
            // HANDS OFF (STICKS CENTERED):
            // Lock attitude using local quaternion error
            let target = hold_state.target_rotation.get_or_insert(transform.rotation);

            let mut delta_rot = transform.rotation.conjugate() * *target;
            if delta_rot.w < 0.0 {
                delta_rot = -delta_rot;
            }

            let (axis, angle) = delta_rot.to_axis_angle();
            let error_vec = if angle.abs() > 1e-5 {
                axis * angle
            } else {
                Vec3::ZERO
            };

            let err_pitch = error_vec.x;
            let err_roll = error_vec.z;

            // Full PD attitude hold + turn compensation
            cmd.pitch = (-err_pitch * P_PITCH + auto_turn_pitch + sensors.local_ang_vel.x * D_PITCH).clamp(-1.0, 1.0);
            cmd.roll = (-err_roll * P_ROLL + sensors.local_ang_vel.z * D_ROLL).clamp(-1.0, 1.0);
        }

        // -----------------------------------------------------------------
        // 3. YAW CHANNEL (Yaw Damper + Top-Rudder Assist + Pilot Pedals)
        // -----------------------------------------------------------------
        cmd.yaw = (yaw_input + auto_top_rudder + sensors.local_ang_vel.y * D_YAW).clamp(-1.0, 1.0);
    }
}