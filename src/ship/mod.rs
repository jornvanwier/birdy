use crate::input::create_input_map;
use crate::ship::throttle::Throttle;
use avian3d::prelude::*;
use bevy::prelude::*;

mod flight;
mod throttle;

pub use flight::{FlightModel, FlightTelemetry};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ship).add_systems(
            Update,
            (
                throttle::handle_throttle.before(flight::apply_flight_forces),
                flight::apply_flight_forces,
            ),
        );
    }
}

#[derive(Component)]
pub struct Ship;

pub fn spawn_ship(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning ship");
    let input_map = create_input_map();

    commands
        .spawn(input_map)
        .insert((
            Name::new("Player"),
            Ship,
            FlightModel::default(),
            FlightTelemetry::default(),
            Throttle::new(0., 0.5, 5.0),
            Transform::from_xyz(0., 0., 10.),
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 0.75, 1.5),
            Mass(800.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                WorldAssetRoot(asset_server.load("game/craft_speederD.glb#Scene0")),
                Transform::from_xyz(-2., -0.4, -1.5),
            ));
            parent.spawn((
                Camera3d::default(),
                // Regular chase cam
                Transform::from_xyz(0.0, 0.5, 6.0),
                // Side view cam
                // Transform::from_xyz(-10.0, 0.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ));
        });
}
