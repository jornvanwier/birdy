use crate::input::create_input_map;
use crate::ship::thrust::Thrust;
use avian3d::prelude::*;
use bevy::prelude::*;

mod flight;
mod thrust;

use crate::ship::flight::{AeroSurface, ControlSurfaceActuator, ControlSurfaceOrientation};
pub use flight::{FlightModel, FlightTelemetry};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_ship, spawn_camera))
            .add_systems(
                Update,
                (
                    thrust::handle_throttle,
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
        area: 0.0,
        lift_slope: 3.5,
        drag_0: 0.02,
        induced_drag_coeff: 0.18,
        stall_angle: 0.43, // ~25 degrees
    };

    let aileron_actuator = ControlSurfaceActuator {
        max_angle: f32::to_radians(8.0),
        speed: 6.0,
        ..default()
    };

    let fin_rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);

    commands
        .spawn_scene(bsn! {
            Name::new("Player")
            Ship
            Visibility::default()
            FlightTelemetry::default()
            template_value(LinearVelocity(Vec3::NEG_Z * 300.0))
            Thrust::new(1.0, 140_000., 0.5, 5.0)
            Transform::from_xyz(0.0, 15.0, 10.0)
            template_value(RigidBody::Dynamic)
            Collider::cuboid(7.0, 3.0, 9.0)
            Mass(12_000.0)
            TransformInterpolation

            Children [
                // --- MAIN WINGS (Fixed) ---
                (
                    #LeftWing
                    template_value(AeroSurface { area: 12.0, ..base_aero })
                    Mesh3d(asset_value(Cuboid::new(3.0, 0.1, 4.0)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.35, 0.4, 0.45)))
                    Transform::from_xyz(-3.0, 0.0, 0.35)
                )
                (
                    #RightWing
                    template_value(AeroSurface { area: 12.0, ..base_aero })
                    Mesh3d(asset_value(Cuboid::new(3.0, 0.1, 4.0)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.35, 0.4, 0.45)))
                    Transform::from_xyz(3.0, 0.0, 0.35)
                ),

                // --- AILERONS (Roll Control - Cyan) ---
                (
                    #LeftAileron
                    template_value(AeroSurface { area: 4.0, ..base_aero })
                    template_value(ControlSurfaceOrientation::Roll { negate: false })
                    template_value(aileron_actuator)
                    Mesh3d(asset_value(Cuboid::new(1.5, 0.12, 1.4)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.2, 0.8, 0.9)))
                    Transform::from_xyz(-4.5, 0.0, 0.8)
                )
                (
                    #RightAileron
                    template_value(AeroSurface { area: 4.0, ..base_aero })
                    template_value(ControlSurfaceOrientation::Roll { negate: true })
                    template_value(aileron_actuator)
                    Mesh3d(asset_value(Cuboid::new(1.5, 0.12, 1.4)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.2, 0.8, 0.9)))
                    Transform::from_xyz(4.5, 0.0, 0.8)
                ),

                // --- HORIZONTAL TAIL & ELEVATOR (Pitch Control - Orange) ---
                (
                    #HorizontalStabilizer
                    template_value(AeroSurface { area: 3.0, ..base_aero })
                    Mesh3d(asset_value(Cuboid::new(3.5, 0.1, 1.2)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.35, 0.4, 0.45)))
                    Transform::from_xyz(0.0, 0.8, 5.0)
                )
                (
                    #Elevator
                    template_value(AeroSurface { area: 3.5, ..base_aero })
                    template_value(ControlSurfaceOrientation::Pitch)
                    ControlSurfaceActuator {
                        max_angle: f32::to_radians(25.0),
                        speed: 5.0,
                    }
                    Mesh3d(asset_value(Cuboid::new(3.5, 0.12, 0.8)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.95, 0.5, 0.1)))
                    Transform::from_xyz(0.0, 0.8, 5.8)
                ),

                // --- VERTICAL TAIL FIN & RUDDER (Yaw Control - Magenta) ---
                (
                    #VerticalStabilizer
                    template_value(AeroSurface { area: 3.0, ..base_aero })
                    Mesh3d(asset_value(Cuboid::new(2.0, 0.1, 1.5)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.35, 0.4, 0.45)))
                    template_value(Transform::from_xyz(0.0, 1.2, 5.2).with_rotation(fin_rot))
                ),
                (
                    #Rudder
                    template_value(AeroSurface { area: 2.0, ..base_aero })
                    template_value(ControlSurfaceOrientation::Yaw)
                    ControlSurfaceActuator {
                        max_angle: f32::to_radians(25.0),
                        speed: 5.0,
                        base_rotation: fin_rot,
                    }
                    Mesh3d(asset_value(Cuboid::new(2.0, 0.12, 0.8)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.9, 0.2, 0.6)))
                    template_value(Transform::from_xyz(0.0, 1.2, 6.0).with_rotation(fin_rot))
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
