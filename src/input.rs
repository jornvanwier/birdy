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
        .with_axis(Action::Throttle, VirtualAxis::new(GamepadButton::DPadDown, GamepadButton::DPadUp))
        .with_axis(Action::Throttle, VirtualAxis::new(KeyCode::ControlLeft, KeyCode::ShiftLeft))
}
