use avian3d::prelude::*;
use bevy::app::{App, Plugin, Startup};
use bevy::asset::AssetServer;
use bevy::camera::Camera3d;
use bevy::prelude::*;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ship);
    }
}

#[derive(Component)]
pub struct Ship;

pub fn spawn_ship(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Name::new("Player"),
            Ship,
            Transform::from_xyz(0., 0., 10.),
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 0.75, 1.5),

            Mass(10.0),
            LinearDamping(0.8),
            AngularDamping(1.0), // Prevents tiny continuous micro-rotations
        ))
        .with_children(|parent| {
            parent.spawn((
                WorldAssetRoot(asset_server.load("game/craft_speederD.glb#Scene0")),
                Transform::from_xyz(-2., -0.4, -1.5),
            ));
            parent.spawn((Camera3d::default(), Transform::from_xyz(0.0, 0.5, 6.0)));
        });
}
