use crate::ship::FlightTelemetry;
use bevy::prelude::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_systems(Update, update_ui);
    }
}

/// Marker enum attached to UI text nodes to identify which field to display
#[derive(Component, Clone, Default, FromTemplate)]
pub enum TelemetryField {
    #[default]
    Speed,
    Thrust,
    Lift,
    Drag,
    AngleOfAttack,
    DynamicPressure,
    Rotation,
}

pub fn setup_ui(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Camera2d
        Node {
            position_type: PositionType::Absolute,
            top: px(10.0),
            right: px(10.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(4.0),
            padding: UiRect::all(px(8.0)),
        }
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6))
        Children [
            (
                Text("FLIGHT TELEMETRY")
                TextFont { font_size: px(16.0) }
                TextColor(Color::WHITE)
            ),
            (
                Text("Speed: 0.00 m/s")
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::Speed
            ),
            (
                Text("Thrust: 0.00 N")
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::Thrust
            ),
            (
                Text("Lift: 0.00 N")
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::Lift
            ),
            (
                Text("Drag: 0.00 N")
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::Drag
            ),
            (
                Text("AoA: 0.00 rad")
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::AngleOfAttack
            ),
            (
                Text("Dyn Press: 0.00 Pa")
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::DynamicPressure
            ),
            (
                Text("Rotation: 0.00 rad/s")
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::Rotation
            ),
        ]
    });
}

pub fn update_ui(
    telemetry_query: Query<&FlightTelemetry, Changed<FlightTelemetry>>,
    mut text_query: Query<(&mut Text, &TelemetryField)>,
) {
    // Only run if telemetry updated this frame
    let Ok(telemetry) = telemetry_query.single() else {
        return;
    };

    for (mut text, field) in &mut text_query {
        text.0 = match field {
            TelemetryField::Speed => {
                format!("Speed: {:.2} m/s", telemetry.linear_velocity.length())
            }
            TelemetryField::Thrust => {
                format!("Thrust: {:.2} N", telemetry.thrust.length())
            }
            TelemetryField::Lift => {
                format!("Lift: {:.2} N", telemetry.lift.length())
            }
            TelemetryField::Drag => {
                format!("Drag: {:.2} N", telemetry.drag.length())
            }
            TelemetryField::AngleOfAttack => format!(
                "AoA: {:.2} rad ({:.1}°)",
                telemetry.angle_of_attack,
                telemetry.angle_of_attack.to_degrees()
            ),
            TelemetryField::DynamicPressure => {
                format!("Dyn Press: {:.2} Pa", telemetry.dynamic_pressure)
            }
            TelemetryField::Rotation => {
                format!("Rotation: {:.2} rad/s", telemetry.angular_velocity.length())
            }
        };
    }
}
