use crate::input::Action;
use crate::ship::Ship;
use avian3d::prelude::forces::ForcesItem;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FlightModel {
    pub max_thrust: f32,         // Engine thrust in Newtons (N)
    pub wing_area: f32,          // Wing surface area (m^2)
    pub lift_coefficient: f32,   // Base lift coefficient (C_L)
    pub lift_slope: f32,         // Rate of C_L gain per radian of AoA (C_L_alpha)
    pub drag_coefficient: f32,   // Base drag coefficient (C_D)
    pub induced_drag_coeff: f32, // Extra drag when unaligned with velocity

    /// Center of Lift offset in local space relative to Center of Mass (+Z is behind CoM)
    pub center_of_lift_offset: Vec3,
    /// Center of Thrust offset in local space relative to Center of Mass (+Z is behind CoM)
    pub center_of_thrust_offset: Vec3,

    /// Tail fin location relative to Center of Mass (+Z is behind CoM)
    pub tail_offset: Vec3,

    /// Surface area of the tail fin (provides passive weathercocking stability)
    pub tail_fin_area: f32,

    pub aileron_wing_position: f32,

    /// Control surface force multipliers
    pub elevator_authority: f32,
    pub rudder_authority: f32,
    pub aileron_authority: f32,

    /// Rotational damping factors in local space (prevents endless spin/oscillations)
    pub angular_damping: Vec3,
}

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

impl Default for FlightModel {
    fn default() -> Self {
        Self {
            max_thrust: 140_000.0,
            wing_area: 32.0,
            lift_coefficient: 0.02,
            lift_slope: 3.5,
            drag_coefficient: 0.02,
            induced_drag_coeff: 0.18,
            center_of_lift_offset: Vec3::new(0.0, 0.0, 0.35),
            center_of_thrust_offset: Vec3::new(0.0, 0.0, 5.),
            tail_offset: Vec3::new(0.0, 0.8, 5.5),
            tail_fin_area: 5.0,
            aileron_wing_position: 4.0,
            elevator_authority: 1.5,
            rudder_authority: 1.5,
            aileron_authority: 1.2,
            angular_damping: Vec3::new(2.0, 2.5, 4.0),
        }
    }
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

#[derive(Component, Reflect, Clone, Default)]
pub enum ControlSurfaceOrientation {
    #[default]
    Pitch,
    Roll {
        negate: bool,
    },
    Yaw,
}
pub fn set_control_surface_targets(
    ships: Query<(&ActionState<Action>, &Children), With<Ship>>,
    mut actuators: Query<(&ControlSurfaceOrientation, &mut ControlSurfaceActuator)>,
) {
    for (action_state, children) in &ships {
        let Vec2 {
            x: roll_input,
            y: pitch_input,
        } = action_state.clamped_axis_pair(&Action::RollPitch);
        let yaw_input = action_state.value(&Action::Yaw);

        for child in children.iter() {
            if let Ok((orientation, mut actuator)) = actuators.get_mut(child) {
                actuator.target_deflection = match *orientation {
                    ControlSurfaceOrientation::Pitch => pitch_input,
                    ControlSurfaceOrientation::Roll { negate } => {
                        roll_input * if negate { -1. } else { 1. }
                    }
                    ControlSurfaceOrientation::Yaw => yaw_input,
                };
            }
        }
    }
}

#[derive(Component, Reflect, Copy, Clone)]
pub struct ControlSurfaceActuator {
    pub max_angle: f32,         // Max deflection in radians
    pub speed: f32,             // Radians per second
    pub target_deflection: f32, // -1.0 to 1.0 from input
    pub base_rotation: Quat,    // Resting orientation (e.g. 90 deg Z for vertical tail)
}

impl Default for ControlSurfaceActuator {
    fn default() -> Self {
        Self {
            max_angle: f32::to_radians(25.0),
            speed: 5.0,
            target_deflection: 0.0,
            base_rotation: Quat::IDENTITY,
        }
    }
}

pub fn update_control_surfaces(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &ControlSurfaceActuator)>,
) {
    for (mut transform, actuator) in query.iter_mut() {
        let target_angle = actuator.target_deflection * actuator.max_angle;
        let deflection_rot = Quat::from_rotation_x(target_angle);
        let target_rot = actuator.base_rotation * deflection_rot;

        transform.rotation = transform
            .rotation
            .rotate_towards(target_rot, actuator.speed * time.delta_secs());
    }
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
