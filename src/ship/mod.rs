use crate::input::create_input_map;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_hanabi::{EffectProperties, ParticleEffect};
use control_surface::{ControlSurfaceActuator, ControlSurfaceOrientation};

mod aero;
mod control_surface;
mod thrust;
mod thrust_fx;

use crate::ship::aero::FuselageDrag;
pub use aero::{AeroSurface, FlightTelemetry};
pub use thrust::Thrust;
use thrust_fx::{ThrusterEffectHandle, ThrusterParticle};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (thrust_fx::setup_thruster_effect, spawn_ship).chain())
            .add_systems(
                Update,
                (
                    thrust::handle_throttle,
                    control_surface::set_control_surface_targets,
                    control_surface::update_control_surfaces
                        .after(control_surface::set_control_surface_targets),
                    thrust_fx::update_thrust_particles.after(thrust::handle_throttle),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    aero::calculate_aero_surface_forces,
                    aero::calculate_fuselage_drag,
                    thrust::apply_thrust,
                ),
            );
    }
}

#[derive(Component, Clone, Default)]
pub struct Ship;

pub fn spawn_ship(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    effect_handle: Res<ThrusterEffectHandle>, // Pass Res by value
) {
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
                (
                    FuselageDrag
                ),

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

            for x_offset in [-1., 1.] {
                // Spawn ParticleEffect here in standard ECS instead of inside bsn!
                parent.spawn((
                    Name::new("JetExhaust"),
                    ThrusterParticle,
                    ParticleEffect::new(effect_handle.0.clone()),
                    EffectProperties::default(),
                    Transform::from_xyz(-x_offset * 1.5, 0.0, 4.2),
                ));
            }
        });
}
