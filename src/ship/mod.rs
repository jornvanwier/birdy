use crate::input::create_input_map;
use crate::ship::throttle::Throttle;
use avian3d::prelude::*;
use bevy::app::{App, Plugin, Startup};
use bevy::asset::AssetServer;
use bevy::camera::Camera3d;
use bevy::prelude::*;

mod throttle;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ship)
            .add_systems(Update, throttle::handle_throttle);
    }
}

#[derive(Component)]
pub struct FlightModel {
    pub wing_area: f32,        // Surface area in m^2
    pub lift_coefficient: f32, // Base C_L
    pub drag_coefficient: f32, // Base C_D
    pub max_thrust: f32,       // Engine thrust in Newtons
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
            Throttle::new(0., 0.5, 5.0),
            Transform::from_xyz(0., 0., 10.),
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 0.75, 1.5),
        ))
        .with_children(|parent| {
            parent.spawn((
                WorldAssetRoot(asset_server.load("game/craft_speederD.glb#Scene0")),
                Transform::from_xyz(-2., -0.4, -1.5),
            ));
            parent.spawn((Camera3d::default(), Transform::from_xyz(0.0, 0.5, 6.0)));
        });
}
