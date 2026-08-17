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
    mut ships: Query<(Forces, &GlobalTransform, &Children)>,
    surfaces: Query<(&AeroSurface, &GlobalTransform)>,
) {
    let air_density = 1.225; // kg/m^3 (sea level standard)

    for (mut forces, ship_transform, children) in ships.iter_mut() {
        let ship_pos = ship_transform.translation();
        let ship_lin_vel = forces.linear_velocity();
        let ship_ang_vel = forces.angular_velocity();

        let mut total_lift = Vec3::ZERO;
        let mut total_drag = Vec3::ZERO;

        for child in children.iter() {
            let Ok((surface, surface_transform)) = surfaces.get(child) else {
                continue;
            };

            let surf_pos = surface_transform.translation();
            let surf_rot = surface_transform.compute_transform().rotation;
            let surf_up = surf_rot * Vec3::Y; // Lift direction / normal to wing

            // Calculate local velocity of this surface through the air
            // v_point = v_linear + omega x r
            let arm = surf_pos - ship_pos;
            let point_velocity = ship_lin_vel + ship_ang_vel.cross(arm);
            let speed = point_velocity.length();

            if speed < 0.1 {
                continue;
            }

            let vel_dir = point_velocity / speed;
            let dynamic_pressure = calculate_dynamic_pressure(air_density, speed);

            // Angle of Attack (AoA) in the surface's local pitch plane
            let local_v = surf_rot.inverse() * point_velocity;
            // local_v.z is backwards (-forward), local_v.y is normal (up)
            let raw_aoa = (-local_v.y).atan2(-local_v.z);
            let aoa = raw_aoa.clamp(-surface.stall_angle, surface.stall_angle);

            // Lift and Drag coefficients
            let cl = aoa * surface.lift_slope;
            let cd = surface.drag_0 + surface.induced_drag_coeff * cl.powi(2);

            // Force Magnitudes
            let lift_mag = dynamic_pressure * surface.area * cl;
            let drag_mag = dynamic_pressure * surface.area * cd;

            // Force Vectors
            // Lift acts perpendicular to relative velocity in the chord-normal plane
            let lift_dir = (surf_up - vel_dir * surf_up.dot(vel_dir)).normalize_or_zero();
            let lift_force = lift_dir * lift_mag;

            // Drag acts opposite to velocity
            let drag_force = -vel_dir * drag_mag;

            let total_surface_force = lift_force + drag_force;

            // Apply to parent rigid body at world offset
            forces.apply_force(total_surface_force);
            forces.apply_torque(arm.cross(total_surface_force));

            total_lift += lift_force;
            total_drag += drag_force;
        }
    }
}

fn calculate_dynamic_pressure(air_density: f32, speed: f32) -> f32 {
    0.5 * air_density * speed.powi(2)
}

#[derive(Component, Reflect, Clone, Copy)]
pub struct FuselageDrag {
    /// Effective drag area (Cd * Area) for forward/backward flight
    /// Streamlined shape: low value (e.g., 0.05 - 0.2 m²)
    pub forward_area: f32,

    /// Effective drag area for lateral/side-slip flight (X-axis)
    /// Bluff cylinder shape: high value (e.g., 1.5 - 4.0 m²)
    pub side_area: f32,

    /// Effective drag area for vertical/belly flight (Y-axis)
    /// Bluff cylinder shape: high value (e.g., 2.0 - 5.0 m²)
    pub top_area: f32,
}

impl Default for FuselageDrag {
    fn default() -> Self {
        Self {
            forward_area: 0.08, // Low resistance cutting through air
            side_area: 2.5,     // High resistance to sideways drifting
            top_area: 3.5,      // High resistance to vertical pancake motion
        }
    }
}

pub fn calculate_fuselage_drag(
    mut ships: Query<(Forces, &GlobalTransform, &FuselageDrag)>,
) {
    let air_density = 1.225; // kg/m^3

    for (mut forces, transform, fuselage) in ships.iter_mut() {
        let rotation = transform.compute_transform().rotation;
        let lin_vel = forces.linear_velocity();

        // 1. Transform world velocity into local aircraft space
        // (X = Right, Y = Up, Z = Backwards / -Z = Forward)
        let local_v = rotation.inverse() * lin_vel;

        // 2. Compute dynamic pressure per axis: 0.5 * rho * v * |v|
        // Retaining the sign ensures drag always opposes motion along that axis
        // TODO can we use calculate_dynamic_pressure?
        let q_x = 0.5 * air_density * local_v.x * local_v.x.abs();
        let q_y = 0.5 * air_density * local_v.y * local_v.y.abs();
        let q_z = 0.5 * air_density * local_v.z * local_v.z.abs();

        // 3. Local drag force opposes velocity in each axis: F = -q * (Cd * A)
        let local_drag = Vec3::new(
            -q_x * fuselage.side_area,
            -q_y * fuselage.top_area,
            -q_z * fuselage.forward_area,
        );

        // 4. Convert back to world space and apply
        let world_drag = rotation * local_drag;
        forces.apply_force(world_drag);
    }
}