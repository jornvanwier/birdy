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
    mut cam_velocity: Local<Vec3>,
    mut prev_ship_vel: Local<Vec3>,
    mut smoothed_accel: Local<Vec3>,
    time: Res<Time>,
) {
    const BASE_OFFSET: Vec3 = Vec3::new(0.0, 3.0, 20.0);

    // Spring tuning
    const OMEGA: f32 = 16.0; // Response frequency (stiffness)
    const ZETA: f32 = 0.85; // Damping ratio (0.75-0.85 = subtle overshoot)

    // Per-axis G-force sensitivity (meters of camera shift per m/s^2 of acceleration)
    // Local axes: +X = Right, +Y = Up, -Z = Forward
    const ACCEL_SCALE: Vec3 = Vec3::new(
        0.01, // Lateral (X): very subtle roll/yaw slide
        0.01, // Vertical (Y): slight sink when pulling high Gs
        0.08, // Longitudinal (Z): punchy throttle kick without flying too far back
    );

    // Maximum allowed G-force displacement in meters
    const MAX_G_OFFSET: Vec3 = Vec3::new(
        0.5, // Max 0.5m side slide
        0.5, // Max 1.0m vertical dip in extreme turns
        2.5, // Max 2.5m rearward kick under full afterburner
    );

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

    // 1. Calculate ship acceleration and smooth high-frequency physics noise
    let raw_accel = (ship_vel.0 - *prev_ship_vel) / delta_time;
    let accel_filter = 1.0 - (-25.0 * delta_time).exp();
    *smoothed_accel = smoothed_accel.lerp(raw_accel, accel_filter);
    *prev_ship_vel = ship_vel.0;

    // 2. Transform acceleration into the ship's local frame
    let local_accel = ship_transform.rotation.inverse() * *smoothed_accel;

    // Inertial reaction opposes acceleration direction
    let g_displacement = Vec3::new(
        -local_accel.x * ACCEL_SCALE.x,
        -local_accel.y * ACCEL_SCALE.y,
        // -Z is forward: accelerating forward makes local_accel.z negative, so -local_accel.z is positive (+Z = backwards)
        -local_accel.z * ACCEL_SCALE.z,
    )
    .clamp(-MAX_G_OFFSET, MAX_G_OFFSET);

    // 3. Compute dynamic target position with the G-force offset applied
    let target_offset = BASE_OFFSET + g_displacement;
    let target_world_pos = ship_world_pos + (ship_transform.rotation * target_offset);
    let target_rotation = ship_transform.rotation;

    // 4. Warp safeguard
    let distance = cam_world_pos.distance(target_world_pos);
    if distance > 1_000.0 {
        let (new_cell, new_translation) =
            grid.translation_to_grid(target_world_pos.as_dvec3() + cam_cell_offset);
        **cam_cell = new_cell;
        cam_transform.translation = new_translation;
        cam_transform.rotation = target_rotation;
        *cam_velocity = ship_vel.0;
        return;
    }

    // 5. Spring-Damper simulation
    let displacement = target_world_pos - cam_world_pos;
    let rel_velocity = *cam_velocity - ship_vel.0;
    let spring_acc = (OMEGA * OMEGA * displacement) - (2.0 * ZETA * OMEGA * rel_velocity);

    *cam_velocity += spring_acc * delta_time;
    let new_cam_world_pos = cam_world_pos + *cam_velocity * delta_time;

    // 6. Deconstruct continuous world position into CellCoord + local Transform
    let (new_cell, new_translation) =
        grid.translation_to_grid(new_cam_world_pos.as_dvec3() + cam_cell_offset);

    **cam_cell = new_cell;
    cam_transform.translation = new_translation;
    cam_transform.rotation = target_rotation;
}
