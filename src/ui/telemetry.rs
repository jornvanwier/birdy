use crate::ship::FlightTelemetry;
use bevy::prelude::*;

pub fn setup_telemetry(mut commands: Commands) {
    commands.spawn_scene(bsn! {
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
                Text()
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::Speed
            ),
            (
                Text()
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::Thrust
            ),
            (
                Text()
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::Lift
            ),
            (
                Text()
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::Drag
            ),
            (
                Text()
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::AngleOfAttack
            ),
            (
                Text()
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::DynamicPressure
            ),
            (
                Text()
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::Rotation
            ),
            (
                Text()
                TextFont { font_size: px(14.0) }
                TextColor(Color::srgb(0.2, 1.0, 0.4))
                TelemetryField::GForce
            ),
        ]
    });
}

pub fn update_telemetry(
    telemetry_query: Query<&FlightTelemetry, Changed<FlightTelemetry>>,
    mut text_query: Query<(&mut Text, &TelemetryField)>,
) {
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
            TelemetryField::GForce => {
                format!(
                    "Gs: {:.2}G Y: ({:.2}G)",
                    telemetry.g_force.length(),
                    telemetry.g_force.y
                )
            }
        };
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
    GForce,
}