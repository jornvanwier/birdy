use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    #[actionlike(Axis)]
    Throttle,
    #[actionlike(DualAxis)]
    RollPitch,
    #[actionlike(Axis)]
    Yaw,
}

pub fn create_input_map() -> InputMap<Action> {
    InputMap::<Action>::default()
        // Throttle: Shift (Up) / Ctrl (Down) or DPad Up/Down
        .with_axis(
            Action::Throttle,
            VirtualAxis::new(KeyCode::ControlLeft, KeyCode::ShiftLeft),
        )
        .with_axis(
            Action::Throttle,
            VirtualAxis::new(GamepadButton::DPadDown, GamepadButton::DPadUp),
        )
        // Roll & Pitch: WASD, Arrow Keys, or Left Stick
        .with_dual_axis(Action::RollPitch, VirtualDPad::wasd())
        .with_dual_axis(Action::RollPitch, VirtualDPad::arrow_keys())
        .with_dual_axis(Action::RollPitch, GamepadStick::LEFT)
        // Yaw: Q (Left) / E (Right) or Gamepad Triggers
        .with_axis(
            Action::Yaw,
            VirtualAxis::new(KeyCode::KeyQ, KeyCode::KeyE),
        )
        .with_axis(
            Action::Yaw,
            VirtualAxis::new(GamepadButton::LeftTrigger2, GamepadButton::RightTrigger2),
        )
}