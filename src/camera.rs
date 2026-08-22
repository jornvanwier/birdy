use crate::ship::Ship;
use bevy::camera::{Camera3d, Exposure, PerspectiveProjection, Projection};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::{AtmosphereEnvironmentMapLight, VolumetricFog};
use bevy::pbr::AtmosphereSettings;
use bevy::prelude::*;
use big_space::prelude::*;

pub struct ChaseCameraPlugin;

impl Plugin for ChaseCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, camera_chase_ship);
    }
}

#[derive(Component, Clone, Default)]
pub struct ChaseCamera;

pub fn camera_scene() -> impl Scene {
    bsn! {
        Camera3d
        ChaseCamera
        FloatingOrigin
        CellCoord::default()
        template_value(Projection::Perspective(PerspectiveProjection {
            far: 500_000.0,
            ..default()
        }))
        Exposure::SUNLIGHT
        template_value(Tonemapping::TonyMcMapface)
        AtmosphereSettings
        AtmosphereEnvironmentMapLight

        Transform::from_xyz(0.0, 1503.0, 30.0)

        VolumetricFog {
            step_count: 64,
        }
    }
}

fn camera_chase_ship(
    grid: Single<&Grid, With<BigSpace>>,
    mut camera_query: Single<(&mut CellCoord, &mut Transform), With<ChaseCamera>>,
    ship_query: Single<(&CellCoord, &Transform), (With<Ship>, Without<ChaseCamera>)>,
    time: Res<Time>,
) {
    const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 3.0, 20.0);
    const BASE_STIFFNESS: f32 = 5.0;

    let edge_len = grid.cell_edge_length();
    let (ref mut cam_cell, ref mut cam_transform) = *camera_query;
    let (ship_cell, ship_transform) = *ship_query;

    // 1. Reconstruct continuous world positions (CellCoord + local Transform)
    let ship_cell_offset = Vec3::new(
        ship_cell.x as f32 * edge_len,
        ship_cell.y as f32 * edge_len,
        ship_cell.z as f32 * edge_len,
    );
    let ship_world_pos = ship_cell_offset + ship_transform.translation;

    let cam_cell_offset = Vec3::new(
        cam_cell.x as f32 * edge_len,
        cam_cell.y as f32 * edge_len,
        cam_cell.z as f32 * edge_len,
    );
    let cam_world_pos = cam_cell_offset + cam_transform.translation;

    // 2. Compute target position behind the ship in continuous world space
    let target_world_pos = ship_world_pos + (ship_transform.rotation * CAMERA_OFFSET);
    let target_rotation = ship_transform.rotation;

    // 3. Distance and exponential decay calculation
    let distance = cam_world_pos.distance(target_world_pos);
    let stiffness =
        BASE_STIFFNESS * (1.0 + (5.0 * distance / CAMERA_OFFSET.length()).exp().min(50.0));
    let delta_time = time.delta_secs();
    let decay = (1.0 - (-stiffness * delta_time).exp()).clamp(0.0, 1.0);

    // 4. Smoothly interpolate in continuous space
    let new_cam_world_pos = cam_world_pos.lerp(target_world_pos, decay);

    // 5. Deconstruct continuous world position into CellCoord + local Transform offset
    let (new_cell, new_translation) = grid.imprecise_translation_to_grid(new_cam_world_pos);

    **cam_cell = new_cell;
    cam_transform.translation = new_translation;
    cam_transform.rotation = cam_transform.rotation.slerp(target_rotation, decay);
}
