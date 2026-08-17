use crate::ship::Ship;
use bevy::prelude::*;

const HUD_COLOR: Color = Color::srgb(0.2, 1.0, 0.4);
const PITCH_PIXELS_PER_DEG: f32 = 8.0;
const MAX_PITCH_LADDER_RADIUS: f32 = 180.0;
const HEADING_PIXELS_PER_DEG: f32 = 6.0;

/// Custom Gizmo group dedicated to HUD drawing
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct HudGizmos;

pub fn draw_attitude_hud(
    mut gizmos: Gizmos<HudGizmos>,
    ship_query: Query<&Transform, With<Ship>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Ship>)>,
) {
    let Ok(ship_transform) = ship_query.single() else {
        return;
    };
    let Ok(cam_transform) = camera_query.single() else {
        return;
    };

    // Helper: projects 2D screen pixel coordinates to a plane 1 meter in front of the 3D camera
    const HUD_DISTANCE: f32 = 1.0;
    const PIXEL_SCALE: f32 = 0.0013; // Scales pixel units to fit within camera FOV
    let to_world = |p: Vec2| -> Vec3 {
        cam_transform.transform_point(Vec3::new(
            p.x * PIXEL_SCALE,
            p.y * PIXEL_SCALE,
            -HUD_DISTANCE,
        ))
    };

    // 1. Decompose rotation into Euler angles
    let (yaw, pitch, roll) = ship_transform.rotation.to_euler(EulerRot::YXZ);
    let bank = -roll;
    let heading_deg = (-yaw).to_degrees().rem_euclid(360.0);
    let hud_rot = Rot2::radians(bank);

    // 2. Center Reticle
    draw_boresight(&mut gizmos, &to_world, cam_transform);

    // 3. Artificial Horizon & Pitch Ladder
    draw_pitch_ladder(&mut gizmos, &to_world, pitch, hud_rot);

    // 4. Roll / Bank Indicator (Sky Pointer)
    draw_roll_indicator(&mut gizmos, &to_world, bank);

    // 5. Yaw / Heading Tape (Top of HUD)
    draw_heading_tape(&mut gizmos, &to_world, heading_deg);
}

/// Fixed aircraft reference symbol at screen center
fn draw_boresight(
    gizmos: &mut Gizmos<HudGizmos>,
    to_world: &impl Fn(Vec2) -> Vec3,
    cam_transform: &Transform,
) {
    let color = HUD_COLOR;
    const PIXEL_SCALE: f32 = 0.0013;

    // Central circle oriented towards the camera
    gizmos.circle(
        Isometry3d::new(to_world(Vec2::ZERO), cam_transform.rotation),
        3.0 * PIXEL_SCALE,
        color,
    );

    // Left and right wing reference bars
    gizmos.line(
        to_world(Vec2::new(-35.0, 0.0)),
        to_world(Vec2::new(-15.0, 0.0)),
        color,
    );
    gizmos.line(
        to_world(Vec2::new(15.0, 0.0)),
        to_world(Vec2::new(35.0, 0.0)),
        color,
    );

    // Top vertical tick
    gizmos.line(
        to_world(Vec2::new(0.0, 10.0)),
        to_world(Vec2::new(0.0, 20.0)),
        color,
    );
}

/// Pitch ladder with rotating horizon line and climb/dive rungs
fn draw_pitch_ladder(
    gizmos: &mut Gizmos<HudGizmos>,
    to_world: &impl Fn(Vec2) -> Vec3,
    current_pitch: f32,
    hud_rot: Rot2,
) {
    let color = HUD_COLOR;
    let current_pitch_deg = current_pitch.to_degrees();

    // Horizon line (0 degrees)
    let horizon_y = -current_pitch_deg * PITCH_PIXELS_PER_DEG;
    if horizon_y.abs() <= MAX_PITCH_LADDER_RADIUS + 40.0 {
        let h_left = hud_rot * Vec2::new(-120.0, horizon_y);
        let h_mid_left = hud_rot * Vec2::new(-40.0, horizon_y);
        let h_mid_right = hud_rot * Vec2::new(40.0, horizon_y);
        let h_right = hud_rot * Vec2::new(120.0, horizon_y);

        gizmos.line(to_world(h_left), to_world(h_mid_left), color);
        gizmos.line(to_world(h_mid_right), to_world(h_right), color);
    }

    // Pitch rungs every 5 degrees from -90 to +90
    for deg in (-18..=18).map(|i| i * 5) {
        if deg == 0 {
            continue;
        }

        let rung_pitch_deg = deg as f32;
        let delta_deg = rung_pitch_deg - current_pitch_deg;
        let y_offset = delta_deg * PITCH_PIXELS_PER_DEG;

        if y_offset.abs() > MAX_PITCH_LADDER_RADIUS {
            continue;
        }

        let is_climb = deg > 0;
        let gap = 35.0;
        let width = if deg % 10 == 0 { 40.0 } else { 22.0 };
        let tick_len = 6.0;
        let tick_dir = if is_climb { -1.0 } else { 1.0 };

        if is_climb {
            let l_outer = hud_rot * Vec2::new(-gap - width, y_offset);
            let l_inner = hud_rot * Vec2::new(-gap, y_offset);
            let l_tick = hud_rot * Vec2::new(-gap - width, y_offset + tick_dir * tick_len);

            let r_inner = hud_rot * Vec2::new(gap, y_offset);
            let r_outer = hud_rot * Vec2::new(gap + width, y_offset);
            let r_tick = hud_rot * Vec2::new(gap + width, y_offset + tick_dir * tick_len);

            gizmos.line(to_world(l_outer), to_world(l_inner), color);
            gizmos.line(to_world(l_outer), to_world(l_tick), color);
            gizmos.line(to_world(r_inner), to_world(r_outer), color);
            gizmos.line(to_world(r_outer), to_world(r_tick), color);
        } else {
            let seg = width / 3.0;
            for i in 0..2 {
                let lx1 = -gap - width + (i as f32 * seg * 1.5);
                let lx2 = lx1 + seg;
                gizmos.line(
                    to_world(hud_rot * Vec2::new(lx1, y_offset)),
                    to_world(hud_rot * Vec2::new(lx2, y_offset)),
                    color,
                );

                let rx1 = gap + (i as f32 * seg * 1.5);
                let rx2 = rx1 + seg;
                gizmos.line(
                    to_world(hud_rot * Vec2::new(rx1, y_offset)),
                    to_world(hud_rot * Vec2::new(rx2, y_offset)),
                    color,
                );
            }

            let l_outer = hud_rot * Vec2::new(-gap - width, y_offset);
            let l_tick = hud_rot * Vec2::new(-gap - width, y_offset + tick_dir * tick_len);
            let r_outer = hud_rot * Vec2::new(gap + width, y_offset);
            let r_tick = hud_rot * Vec2::new(gap + width, y_offset + tick_dir * tick_len);

            gizmos.line(to_world(l_outer), to_world(l_tick), color);
            gizmos.line(to_world(r_outer), to_world(r_tick), color);
        }
    }
}

/// Sky Pointer
fn draw_roll_indicator(
    gizmos: &mut Gizmos<HudGizmos>,
    to_world: &impl Fn(Vec2) -> Vec3,
    current_bank: f32,
) {
    let color = HUD_COLOR;
    let radius = 160.0;
    let arc_center = Vec2::ZERO;

    let tick_angles_deg = [
        -60.0f32, -45.0, -30.0, -20.0, -10.0, 0.0, 10.0, 20.0, 30.0, 45.0, 60.0,
    ];

    for &bank_deg in &tick_angles_deg {
        let angle = std::f32::consts::FRAC_PI_2 - bank_deg.to_radians();
        let dir = Vec2::new(angle.cos(), angle.sin());
        let tick_len = if bank_deg == 0.0 || bank_deg.abs() == 30.0 || bank_deg.abs() == 60.0 {
            12.0
        } else {
            6.0
        };

        gizmos.line(
            to_world(arc_center + dir * radius),
            to_world(arc_center + dir * (radius + tick_len)),
            color,
        );
    }

    let sky_angle = std::f32::consts::FRAC_PI_2 + current_bank;
    let p_dir = Vec2::new(sky_angle.cos(), sky_angle.sin());
    let p_tangent = Vec2::new(-p_dir.y, p_dir.x);

    let tip = arc_center + p_dir * (radius - 2.0);
    let base_left = arc_center + p_dir * (radius - 12.0) + p_tangent * 5.0;
    let base_right = arc_center + p_dir * (radius - 12.0) - p_tangent * 5.0;

    gizmos.line(to_world(tip), to_world(base_left), color);
    gizmos.line(to_world(base_left), to_world(base_right), color);
    gizmos.line(to_world(base_right), to_world(tip), color);
}

/// Horizontal heading/yaw tape
fn draw_heading_tape(
    gizmos: &mut Gizmos<HudGizmos>,
    to_world: &impl Fn(Vec2) -> Vec3,
    current_heading_deg: f32,
) {
    let color = HUD_COLOR;
    let tape_y = 230.0;
    let tape_half_width = 140.0;

    gizmos.line(
        to_world(Vec2::new(-tape_half_width, tape_y)),
        to_world(Vec2::new(tape_half_width, tape_y)),
        color,
    );

    let pointer_tip = Vec2::new(0.0, tape_y - 2.0);
    gizmos.line(
        to_world(Vec2::new(-6.0, tape_y + 8.0)),
        to_world(pointer_tip),
        color,
    );
    gizmos.line(
        to_world(Vec2::new(6.0, tape_y + 8.0)),
        to_world(pointer_tip),
        color,
    );

    let start_tick = ((current_heading_deg - 30.0) / 5.0).floor() as i32 * 5;
    let end_tick = ((current_heading_deg + 30.0) / 5.0).ceil() as i32 * 5;

    for tick in (start_tick..=end_tick).step_by(5) {
        let tick_deg = (tick as f32).rem_euclid(360.0);
        let mut delta = tick_deg - current_heading_deg;
        if delta > 180.0 {
            delta -= 360.0;
        } else if delta < -180.0 {
            delta += 360.0;
        }

        let x = delta * HEADING_PIXELS_PER_DEG;
        if x.abs() <= tape_half_width {
            let is_major = (tick % 10) == 0;
            let tick_height = if is_major { 10.0 } else { 5.0 };
            gizmos.line(
                to_world(Vec2::new(x, tape_y)),
                to_world(Vec2::new(x, tape_y - tick_height)),
                color,
            );
        }
    }
}
