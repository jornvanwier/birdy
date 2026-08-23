use avian3d::prelude::forces::ForcesItem;
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

pub struct Kinematics {
    rotation: Quat,
    linear_velocity: Vec3,
    angular_velocity: Vec3,
}

pub fn calculate_aerodynamic_forces(
    mut ships: Query<(
        Forces,
        &Transform,
        &FuselageDrag,
        &Children,
        Option<&mut FlightTelemetry>,
    )>,
    surfaces: Query<(&AeroSurface, &Transform)>,
) {
    let air_density = 1.225; // kg/m^3

    for (mut forces, ship_transform, fuselage, children, mut telemetry) in ships.iter_mut() {
        let ship_kinematics = Kinematics {
            rotation: ship_transform.rotation,
            linear_velocity: forces.linear_velocity(),
            angular_velocity: forces.angular_velocity(),
        };

        let mut total_lift = Vec3::ZERO;
        let mut total_aero_surface_drag = Vec3::ZERO;

        for child in children.iter() {
            let Ok((surface, child_transform)) = surfaces.get(child) else {
                continue;
            };

            if let Some((lift_force, drag_force)) = apply_aero_surface_lift_and_drag(
                air_density,
                &mut forces,
                &ship_kinematics,
                surface,
                child_transform,
            ) {
                total_lift += lift_force;
                total_aero_surface_drag += drag_force;
            };
        }

        let fuselage_drag =
            apply_fuselage_drag(fuselage, &ship_kinematics, air_density, &mut forces);

        if let Some(ref mut t) = telemetry {
            t.linear_velocity = ship_kinematics.linear_velocity;
            t.angular_velocity = ship_kinematics.angular_velocity;
            t.lift = total_lift;
            t.drag = total_aero_surface_drag + fuselage_drag;
        }
    }
}

fn apply_aero_surface_lift_and_drag(
    air_density: f32,
    forces: &mut ForcesItem,
    ship_kinematics: &Kinematics,
    surface: &AeroSurface,
    child_transform: &Transform,
) -> Option<(Vec3, Vec3)> {
    // World-space lever arm and orientation for this aerodynamic surface
    let arm = ship_kinematics.rotation * child_transform.translation;
    let surf_rot = ship_kinematics.rotation * child_transform.rotation;
    let surf_up = surf_rot * Vec3::Y; // Wing normal (lift axis)

    // Surface velocity relative to air: v_point = v_linear + omega x r
    let point_velocity =
        ship_kinematics.linear_velocity + ship_kinematics.angular_velocity.cross(arm);
    let speed = point_velocity.length();

    if speed < 0.1 {
        return None;
    }

    let vel_dir = point_velocity / speed;
    let dynamic_pressure = calculate_scalar_dynamic_pressure(air_density, speed);

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
    Some((lift_force, drag_force))
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

/// Standard scalar dynamic pressure (q = 0.5 * rho * v^2) in Pascals (N/m^2).
/// Used for continuous airflow across lifting surfaces.
#[inline]
fn calculate_scalar_dynamic_pressure(air_density: f32, speed: f32) -> f32 {
    0.5 * air_density * speed.powi(2)
}

/// Signed, component-wise dynamic pressure per body axis (q_i = 0.5 * rho * v_i * |v_i|).
/// Preserves directional sign along each local axis for bluff-body cross-flow drag.
#[inline]
fn calculate_component_dynamic_pressure(air_density: f32, local_velocity: Vec3) -> Vec3 {
    0.5 * air_density * local_velocity * local_velocity.abs()
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

pub fn apply_fuselage_drag(
    fuselage: &FuselageDrag,
    ship_kinematics: &Kinematics,
    air_density: f32,
    forces: &mut ForcesItem,
) -> Vec3 {
    // Transform velocity into local aircraft space (X = Right, Y = Up, -Z = Forward)
    let local_v = ship_kinematics.rotation.inverse() * ship_kinematics.linear_velocity;

    // Component-wise dynamic pressure vector (X = side, Y = top, Z = axial)
    let q_local = calculate_component_dynamic_pressure(air_density, local_v);

    // Reference areas mapped to local axes (X: side, Y: top, Z: forward)
    let areas = Vec3::new(fuselage.side_area, fuselage.top_area, fuselage.forward_area);

    // Drag opposes motion along each respective axis
    let local_drag = -q_local * areas;
    let world_drag = ship_kinematics.rotation * local_drag;

    forces.apply_force(world_drag);
    world_drag
}
