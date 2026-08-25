use crate::environment::air_density::LocalAirDensity;
use crate::ship::Player;
use crate::ship::sensors::{FlightSensorData, LiftAndDragMeasurement, ThrustMeasurement};
use bevy::prelude::*;
use big_space::prelude::CellCoord;

pub fn setup_telemetry(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: px(10.0),
            right: px(10.0),
            width: px(220.0),
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
            telemetry_field(TelemetryField::Speed),
            telemetry_field(TelemetryField::Thrust),
            telemetry_field(TelemetryField::Lift),
            telemetry_field(TelemetryField::Drag),
            telemetry_field(TelemetryField::AngleOfAttack),
            telemetry_field(TelemetryField::AirDensity),
            telemetry_field(TelemetryField::DynamicPressure),
            telemetry_field(TelemetryField::Rotation),
            telemetry_field(TelemetryField::GForce),
            telemetry_field(TelemetryField::LocalPosition),
            telemetry_field(TelemetryField::CellPosition),
        ]
    });
}

fn telemetry_field(field: TelemetryField) -> impl Scene {
    bsn! {
        Text()
        TextFont { font_size: px(14.0) }
        TextColor(Color::srgb(0.2, 1.0, 0.4))
        template_value(field)
    }
}

pub fn update_telemetry(
    telemetry_query: Single<
        (
            &FlightSensorData,
            &LiftAndDragMeasurement,
            &ThrustMeasurement,
            &LocalAirDensity,
            &Transform,
            &CellCoord,
        ),
        With<Player>,
    >,
    mut text_query: Query<(&mut Text, &TelemetryField)>,
) {
    let (
        sensors,
        LiftAndDragMeasurement { lift, drag },
        ThrustMeasurement(thrust),
        air_density,
        transform,
        cell,
    ) = *telemetry_query;

    for (mut text, field) in &mut text_query {
        text.0 = match field {
            TelemetryField::Speed => {
                format!("Speed: {:.2} m/s", sensors.true_airspeed)
            }
            TelemetryField::Thrust => {
                format!("Thrust: {:.2} N", thrust.length())
            }
            TelemetryField::Lift => {
                format!("Lift: {:.2} N", lift.length())
            }
            TelemetryField::Drag => {
                format!("Drag: {:.2} N", drag.length())
            }
            TelemetryField::AngleOfAttack => format!(
                "AoA: {:.2} rad ({:.1}°)",
                sensors.aoa,
                sensors.aoa.to_degrees()
            ),
            TelemetryField::DynamicPressure => {
                format!("Dyn Press: {:.2} Pa", sensors.dynamic_pressure)
            }
            TelemetryField::Rotation => {
                format!("Rotation: {:.2} rad/s", sensors.local_ang_vel.length())
            }
            TelemetryField::GForce => {
                format!(
                    "Gs: {:.2}G Y: ({:.2}G)",
                    sensors.g_force_local.length(),
                    sensors.g_force_local.y
                )
            }
            TelemetryField::LocalPosition => {
                format!(
                    "@ {:.2},{:.2},{:.2}",
                    transform.translation.x, transform.translation.y, transform.translation.z
                )
            }
            TelemetryField::CellPosition => {
                format!("# {},{},{}", cell.x, cell.y, cell.z)
            }
            TelemetryField::AirDensity => {
                format!("{} kg/m^3", air_density.0)
            }
        };
    }
}

/// Marker enum attached to UI text nodes to identify which field to display
#[derive(Component, Clone, Default)]
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
    LocalPosition,
    CellPosition,
    AirDensity,
}
