use crate::ship::Ship;
use avian3d::prelude::LinearVelocity;
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
    ship_query: Single<
        (&CellCoord, &Transform, &LinearVelocity),
        (With<Ship>, Without<ChaseCamera>),
    >,
    mut previous_ship_velocity: Local<Vec3>,
    time: Res<Time>,
) {
    const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 3.0, 20.0);
    // Responsiveness of the camera relaxing back into place (e.g. 4.0 - 8.0)
    const BASE_STIFFNESS: f32 = 6.0;

    let (ref mut cam_cell, ref mut cam_transform) = *camera_query;
    let (ship_cell, ship_transform, ship_vel) = *ship_query;

    let delta_time = time.delta_secs();
    if delta_time <= 0.0 {
        return;
    }

    let cam_cell_offset = grid.cell_to_float(cam_cell);
    let ship_world_pos =
        (grid.grid_position_double(ship_cell, ship_transform) - cam_cell_offset).as_vec3();
    let cam_world_pos =
        (grid.grid_position_double(cam_cell, cam_transform) - cam_cell_offset).as_vec3();

    // 1. Nominal target position in continuous space
    let target_world_pos = ship_world_pos + (ship_transform.rotation * CAMERA_OFFSET);
    let target_rotation = ship_transform.rotation;

    // 2. Warp safeguard for teleportation/respawns
    let distance = cam_world_pos.distance(target_world_pos);
    if distance > 1_000.0 {
        info!("Warping camera");
        let (new_cell, new_translation) =
            grid.translation_to_grid(target_world_pos.as_dvec3() + cam_cell_offset);
        **cam_cell = new_cell;
        cam_transform.translation = new_translation;
        cam_transform.rotation = target_rotation;
        *previous_ship_velocity = ship_vel.0;
        return;
    }

    // 3. Compute acceleration from change in physics velocity
    let current_acceleration = (ship_vel.0 - *previous_ship_velocity) / delta_time;

    // 4. KINEMATIC FEEDFORWARD:
    // Predict where the camera would naturally move by matching the ship's motion:
    // displacement = v * dt + 0.5 * a * dt^2
    let predicted_cam_world_pos = cam_world_pos
        + (ship_vel.0 * delta_time)
        + (0.5 * current_acceleration * delta_time * delta_time);

    // 5. Exponential decay factor
    let decay = 1.0 - (-BASE_STIFFNESS * delta_time).exp();

    // 6. Interpolate from predicted camera position towards target
    let new_cam_world_pos = predicted_cam_world_pos.lerp(target_world_pos, decay);
    let new_cam_rot = cam_transform.rotation.slerp(target_rotation, decay);

    // 7. Deconstruct continuous world position into CellCoord + local Transform
    let (new_cell, new_translation) =
        grid.translation_to_grid(new_cam_world_pos.as_dvec3() + cam_cell_offset);

    **cam_cell = new_cell;
    cam_transform.translation = new_translation;
    cam_transform.rotation = new_cam_rot;

    // 8. Update history
    *previous_ship_velocity = ship_vel.0;
}
