use bevy::prelude::*;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::*;
use virtual_joystick::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VirtualJoystickState>()
            .add_plugins((
                InputManagerPlugin::<Action>::default(),
                VirtualJoystickPlugin::<JoystickId>::default(),
            ))
            .add_systems(Startup, spawn_joystick)
            // 1. Record incoming joystick messages
            .add_systems(Update, record_joystick_messages)
            // 2. Feed axis into Leafwing in PreUpdate (for normal Update systems)
            .add_systems(
                PreUpdate,
                apply_joystick_to_leafwing.after(InputManagerSystem::Update),
            )
            // 3. Feed axis into Leafwing in FixedPreUpdate (for FixedUpdate physics/flight systems)
            .add_systems(
                FixedPreUpdate,
                apply_joystick_to_leafwing.after(InputManagerSystem::Update),
            );
    }
}

#[derive(Default, Debug, Reflect, Hash, Clone, PartialEq, Eq)]
pub enum JoystickId {
    #[default]
    RollPitch,
}

#[derive(Resource, Default)]
pub struct VirtualJoystickState {
    pub roll_pitch: Vec2,
    pub is_active: bool,
}

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
        .with_axis(
            Action::Throttle,
            VirtualAxis::new(KeyCode::ControlLeft, KeyCode::ShiftLeft),
        )
        .with_axis(
            Action::Throttle,
            VirtualAxis::new(GamepadButton::DPadDown, GamepadButton::DPadUp),
        )
        .with_dual_axis(Action::RollPitch, VirtualDPad::wasd())
        .with_dual_axis(Action::RollPitch, VirtualDPad::arrow_keys())
        .with_dual_axis(Action::RollPitch, GamepadStick::LEFT)
        .with_axis(Action::Yaw, VirtualAxis::new(KeyCode::KeyQ, KeyCode::KeyE))
        .with_axis(
            Action::Yaw,
            VirtualAxis::new(GamepadButton::LeftTrigger, GamepadButton::RightTrigger),
        )
}

fn spawn_joystick(mut commands: Commands, asset_server: Res<AssetServer>) {
    create_joystick(
        &mut commands,
        JoystickId::RollPitch,
        asset_server.load("game/Knob.png"),
        asset_server.load("game/Outline.png"),
        None,
        None,
        None,
        Vec2::new(75., 75.),
        Vec2::new(150., 150.),
        Node {
            // Covers only the bottom-right 50% x 50% quadrant of the screen
            width: Val::Percent(50.),
            height: Val::Percent(50.),
            position_type: PositionType::Absolute,
            left: Val::Px(0.),
            bottom: Val::Px(0.),

            ..default()
        },
        JoystickFloating,
        NoAction,
    );
}

fn record_joystick_messages(
    mut reader: MessageReader<VirtualJoystickMessage<JoystickId>>,
    mut stick_state: ResMut<VirtualJoystickState>,
) {
    for joystick in reader.read() {
        if joystick.id() == JoystickId::RollPitch {
            match joystick.get_type() {
                VirtualJoystickMessageType::Up => {
                    stick_state.roll_pitch = Vec2::ZERO;
                    stick_state.is_active = false;
                }
                VirtualJoystickMessageType::Press | VirtualJoystickMessageType::Drag => {
                    stick_state.roll_pitch = *joystick.axis();
                    stick_state.is_active = true;
                }
            }
        }
    }
}

fn apply_joystick_to_leafwing(
    stick_state: Res<VirtualJoystickState>,
    mut action_query: Query<&mut ActionState<Action>>,
) {
    if stick_state.is_active {
        for mut action_state in &mut action_query {
            action_state.set_axis_pair(&Action::RollPitch, stick_state.roll_pitch);
        }
    }
}
