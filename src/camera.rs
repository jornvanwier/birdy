use crate::ship::Ship;
use avian3d::interpolation::TransformInterpolation;
use avian3d::parry::glamx::Vec3;
use bevy::app::App;
use bevy::camera::{Camera3d, Exposure, PerspectiveProjection, Projection};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::AtmosphereEnvironmentMapLight;
use bevy::pbr::AtmosphereSettings;
use bevy::prelude::*;

pub struct ChaseCameraPlugin;

impl Plugin for ChaseCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(FixedUpdate, camera_chase_ship);
    }
}

#[derive(Component, Clone, Default)]
struct ChaseCamera;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        ChaseCamera,
        Projection::Perspective(PerspectiveProjection {
            far: 500_000.0,
            ..default()
        }),
        Exposure::SUNLIGHT,
        Tonemapping::TonyMcMapface,
        AtmosphereSettings::default(),
        AtmosphereEnvironmentMapLight::default(),
        TransformInterpolation,
    ));
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
