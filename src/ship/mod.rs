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
        app.add_systems(Startup, (spawn_ship, spawn_camera))
            .add_systems(
                Update,
                (
                    throttle::handle_throttle,
                    flight::set_control_surface_targets,
                    flight::update_control_surfaces.after(flight::set_control_surface_targets),
                ),
            )
            .add_systems(FixedUpdate, camera_chase_ship)
            .add_systems(FixedUpdate, flight::calculate_aero_surface_forces);
    }
}

#[derive(Component)]
pub struct Ship;

pub fn spawn_ship(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning ship");
    let input_map = create_input_map();

    let asset_scale = 4.;

    commands
        .spawn(input_map)
        .insert((
            Name::new("Player"),
            Ship,
            Visibility::default(),
            FlightModel::default(),
            FlightTelemetry::default(),
            LinearVelocity(Vec3::NEG_Z * 300.),
            Throttle::new(1., 0.5, 5.0),
            Transform::from_xyz(0., 15., 10.),
            RigidBody::Dynamic,
            Collider::cuboid(7.0, 3.0, 9.0),
            Mass(12_000.0),
            // Smooth out movement between physics updates
            TransformInterpolation,
        ))
        .with_children(|parent| {
            parent.spawn((
                WorldAssetRoot(asset_server.load("game/craft_speederD.glb#Scene0")),
                Transform::from_translation(Vec3::new(-2., -0.4, -1.5) * asset_scale)
                    .with_scale(Vec3::splat(asset_scale)),
            ));
        });
}

#[derive(Component, Clone, Default)]
struct ChaseCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn_scene(bsn! {
       Camera3d::default()
        ChaseCamera
        Camera {
            order: 1,
        }
        TransformInterpolation
    });
}

fn camera_chase_ship(
    mut camera_transform: Single<&mut Transform, With<ChaseCamera>>,
    ship_transform: Single<&Transform, (With<Ship>, Without<ChaseCamera>)>,
    time: Res<Time>,
) {
    const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 3.0, 20.0);
    const BASE_STIFFNESS: f32 = 5.0;

    let target_position = ship_transform.transform_point(CAMERA_OFFSET);

    let target_rotation = ship_transform.rotation;

    let true_offset_distance = camera_transform.translation.distance(target_position);
    let translation_stiffness =
        BASE_STIFFNESS * (1.0 + (5.0 * true_offset_distance / CAMERA_OFFSET.length()).exp());

    let delta_time = time.delta_secs();

    let decay = 1.0 - (-translation_stiffness * delta_time).exp();

    camera_transform.translation = camera_transform.translation.lerp(target_position, decay);
    camera_transform.rotation = camera_transform.rotation.slerp(target_rotation, decay);
}
