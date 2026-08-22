use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Default)]
pub struct FlightTelemetry {
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub thrust: Vec3,
    pub drag: Vec3,
    pub lift: Vec3,
    pub angle_of_attack: f32,
    pub slip_ratio: f32,
    pub dynamic_pressure: f32,
    pub g_force: Vec3,
}

#[derive(Component, Reflect, Default, Clone, Copy)]
pub struct AeroSurface {
    /// Surface area in m^2
    pub area: f32,
    /// Rate of C_L gain per radian of AoA (standard thin airfoil is ~2.0 * PI, or ~3.5-5.5)
    pub lift_slope: f32,
    /// Zero-lift parasitic drag coefficient (typically 0.01 - 0.03)
    pub drag_0: f32,
    /// Induced drag factor k (typically ~0.04 - 0.2 depending on aspect ratio)
    pub induced_drag_coeff: f32,
    /// Stall angle in radians (e.g., 0.35 rad ≈ 20°)
    pub stall_angle: f32,
}

pub fn calculate_aero_surface_forces(
    mut ships: Query<(Forces, &Transform, &Children, Option<&mut FlightTelemetry>)>,
    surfaces: Query<(&AeroSurface, &Transform)>,
) {
    let air_density = 1.225; // kg/m^3

    for (mut forces, ship_transform, children, mut telemetry) in ships.iter_mut() {
        let ship_rot = ship_transform.rotation;
        let ship_lin_vel = forces.linear_velocity();
        let ship_ang_vel = forces.angular_velocity();

        let mut total_lift = Vec3::ZERO;
        let mut total_drag = Vec3::ZERO;

        for child in children.iter() {
            let Ok((surface, child_transform)) = surfaces.get(child) else {
                continue;
            };

            // World-space lever arm and orientation for this aerodynamic surface
            let arm = ship_rot * child_transform.translation;
            let surf_rot = ship_rot * child_transform.rotation;
            let surf_up = surf_rot * Vec3::Y; // Wing normal (lift axis)

            // Surface velocity relative to air: v_point = v_linear + omega x r
            let point_velocity = ship_lin_vel + ship_ang_vel.cross(arm);
            let speed = point_velocity.length();

            if speed < 0.1 {
                continue;
            }

            let vel_dir = point_velocity / speed;
            let dynamic_pressure = calculate_dynamic_pressure(air_density, speed);

            // Angle of Attack (AoA) in the surface's local pitch plane
            let local_v = surf_rot.inverse() * point_velocity;
            let raw_aoa = (-local_v.y).atan2(-local_v.z);

            // Aerodynamic Coefficients
            let (cl, cd) = calculate_aerodynamic_coefficients(surface, raw_aoa);

            // Force Magnitudes
            let lift_mag = dynamic_pressure * surface.area * cl;
            let drag_mag = dynamic_pressure * surface.area * cd;

            // Lift is perpendicular to velocity in the wing's lift plane
            let lift_dir = (surf_up - vel_dir * surf_up.dot(vel_dir)).normalize_or_zero();
            let lift_force = lift_dir * lift_mag;
            let drag_force = -vel_dir * drag_mag;

            let total_surface_force = lift_force + drag_force;

            forces.apply_force(total_surface_force);
            forces.apply_torque(arm.cross(total_surface_force));

            total_lift += lift_force;
            total_drag += drag_force;
        }

        if let Some(ref mut t) = telemetry {
            t.linear_velocity = ship_lin_vel;
            t.angular_velocity = ship_ang_vel;
            t.lift = total_lift;
            t.drag = total_drag;
        }
    }
}

fn calculate_aerodynamic_coefficients(surface: &AeroSurface, aoa: f32) -> (f32, f32) {
    const STALL_WIDTH: f32 = 0.08;
    const CD_POST_STALL_PLATE: f32 = 1.5;

    let separation = 1.0 / (1.0 + (-(aoa.abs() - surface.stall_angle) / STALL_WIDTH).exp());

    let cl_attached = surface.lift_slope * aoa;
    let cd_attached = surface.drag_0 + surface.induced_drag_coeff * cl_attached.powi(2);

    let cl_separated = 0.5 * CD_POST_STALL_PLATE * (2.0 * aoa).sin();
    let cd_separated = surface.drag_0 + CD_POST_STALL_PLATE * aoa.sin().powi(2);

    let cl = (1.0 - separation) * cl_attached + separation * cl_separated;
    let cd = (1.0 - separation) * cd_attached + separation * cd_separated;

    (cl, cd)
}

fn calculate_dynamic_pressure(air_density: f32, speed: f32) -> f32 {
    0.5 * air_density * speed.powi(2)
}

#[derive(Component, Reflect, Clone, Copy)]
pub struct FuselageDrag {
    pub forward_area: f32,
    pub side_area: f32,
    pub top_area: f32,
}

impl Default for FuselageDrag {
    fn default() -> Self {
        Self {
            forward_area: 0.08,
            side_area: 2.5,
            top_area: 3.5,
        }
    }
}

pub fn calculate_fuselage_drag(mut ships: Query<(Forces, &Transform, &FuselageDrag)>) {
    let air_density = 1.225; // kg/m^3

    for (mut forces, ship_transform, fuselage) in ships.iter_mut() {
        let rotation = ship_transform.rotation;
        let lin_vel = forces.linear_velocity();

        // Transform world velocity into local aircraft space (X = Right, Y = Up, -Z = Forward)
        let local_v = rotation.inverse() * lin_vel;

        // Dynamic pressure per axis: 0.5 * rho * v * |v|
        let q_x = 0.5 * air_density * local_v.x * local_v.x.abs();
        let q_y = 0.5 * air_density * local_v.y * local_v.y.abs();
        let q_z = 0.5 * air_density * local_v.z * local_v.z.abs();

        let local_drag = Vec3::new(
            -q_x * fuselage.side_area,
            -q_y * fuselage.top_area,
            -q_z * fuselage.forward_area,
        );

        let world_drag = rotation * local_drag;
        forces.apply_force(world_drag);
    }
}
