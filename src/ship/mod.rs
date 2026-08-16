use crate::input::create_input_map;
use crate::ship::throttle::Throttle;
use avian3d::prelude::*;
use bevy::prelude::*;

mod flight;
mod throttle;

use crate::ship::flight::{AeroSurface, ControlSurfaceActuator, ControlSurfaceOrientation};
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

#[derive(Component, Clone, Default)]
pub struct Ship;

pub fn spawn_ship(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning ship");
    let input_map = create_input_map();
    let asset_scale = 4.0;

    let base_aero = AeroSurface {
        area: 12.0,
        lift_slope: 3.5,
        drag_0: 0.02,
        induced_drag_coeff: 0.18,
        stall_angle: 0.43, // ~25 degrees
    };

    let (axis_x, axis_y, axis_z) = (Vec3::X, Vec3::Y, Vec3::Z);

    let fin_rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);

    commands
        .spawn_scene(bsn! {
            Name::new("Player")
            Ship
            Visibility::default()
            FlightTelemetry::default()
            template_value(LinearVelocity(Vec3::NEG_Z * 300.0))
            Throttle::new(1.0, 0.5, 5.0)
            Transform::from_xyz(0.0, 15.0, 10.0)
            template_value(RigidBody::Dynamic)
            Collider::cuboid(7.0, 3.0, 9.0)
            Mass(12_000.0)
            TransformInterpolation

            // Visual 3D Asset
            // (
            //     WorldAssetRoot(asset_server.load("game/craft_speederD.glb#Scene0"))
            //     Transform::from_translation(Vec3::new(-2.0, -0.4, -1.5) * asset_scale)
            //         .with_scale(Vec3::splat(asset_scale))
            // )

            Children [

            // --- MAIN WINGS (Fixed) ---
            (
                #LeftWing
                AeroSurface {
                    area: 12.0,
                    lift_slope: 3.5,
                    drag_0: 0.02,
                    induced_drag_coeff: 0.18,
                    stall_angle: 0.43, // ~25 degrees
                }
                Transform::from_xyz(-3.0, 0.0, 0.35)
            )
            (
                #RightWing
                AeroSurface {
                    area: 12.0,
                    lift_slope: 3.5,
                    drag_0: 0.02,
                    induced_drag_coeff: 0.18,
                    stall_angle: 0.43, // ~25 degrees
                }
                Transform::from_xyz(3.0, 0.0, 0.35)
            ),

            // --- AILERONS (Roll Control) ---
            (
                #LeftAileron
                AeroSurface {
                    area: 4.0,
                    lift_slope: 3.5,
                    drag_0: 0.02,
                    induced_drag_coeff: 0.18,
                    stall_angle: 0.43, // ~25 degrees
                }
                template_value(ControlSurfaceOrientation::Roll { negate: false })
                ControlSurfaceActuator {
                    max_angle: f32::to_radians(20.0),
                    speed: 6.0,
                    target_deflection: 0.0,
                }
                Transform::from_xyz(-4.5, 0.0, 0.8)
            )
            (
                #RightAileron
                AeroSurface {
                    area: 4.0,
                    lift_slope: 3.5,
                    drag_0: 0.02,
                    induced_drag_coeff: 0.18,
                    stall_angle: 0.43, // ~25 degrees
                }
                template_value(ControlSurfaceOrientation::Roll { negate: true })
                ControlSurfaceActuator {
                    max_angle: f32::to_radians(20.0),
                    speed: 6.0,
                    target_deflection: 0.0,
                }
                Transform::from_xyz(4.5, 0.0, 0.8)
            ),

            // --- HORIZONTAL TAIL & ELEVATOR (Pitch Control & Stability) ---
            (
                #HorizontalStabilizer
                AeroSurface {
                    area: 3.0,
                    lift_slope: 3.5,
                    drag_0: 0.02,
                    induced_drag_coeff: 0.18,
                    stall_angle: 0.43, // ~25 degrees
                }
                Transform::from_xyz(0.0, 0.8, 5.0)
            )
            (
                #Elevator
                AeroSurface {
                    area: 3.5,
                    lift_slope: 3.5,
                    drag_0: 0.02,
                    induced_drag_coeff: 0.18,
                    stall_angle: 0.43, // ~25 degrees
                }
                template_value(ControlSurfaceOrientation::Pitch)
                ControlSurfaceActuator {
                    max_angle: f32::to_radians(25.0),
                    speed: 5.0,
                    target_deflection: 0.0,
                }
                Transform::from_xyz(0.0, 0.8, 5.8)
            ),

            // --- VERTICAL TAIL FIN & RUDDER (Yaw Stability & Weathercocking) ---
            // Rotated 90° on Z so wing normal (Vec3::Y) points horizontally along X (Vec3::NEG_X)
            (
                #VerticalStabilizer
                AeroSurface {
                    area: 3.0,
                    lift_slope: 3.5,
                    drag_0: 0.02,
                    induced_drag_coeff: 0.18,
                    stall_angle: 0.43, // ~25 degrees
                }
                template_value(Transform::from_xyz(0.0, 1.2, 5.2)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)))
            ),
            (
                #Rudder
                AeroSurface {
                    area: 2.0,
                    lift_slope: 3.5,
                    drag_0: 0.02,
                    induced_drag_coeff: 0.18,
                    stall_angle: 0.43, // ~25 degrees
                }
                template_value(ControlSurfaceOrientation::Yaw)
                ControlSurfaceActuator {
                    // When rotated 90 deg around Z, local Y is world -X, so rotating around local Y turns the rudder left/right
                    max_angle: f32::to_radians(25.0),
                    speed: 5.0,
                    target_deflection: 0.0,
                base_rotation: fin_rot,
                }
                template_value(Transform::from_xyz(0.0, 1.2, 6.0)
                    .with_rotation(fin_rot))
            )
            ]
        })
        .insert(input_map)
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
