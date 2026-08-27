use crate::input::create_input_map;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_hanabi::{EffectProperties, ParticleEffect};
use big_space::prelude::{BigSpace, CellCoord};
use control_surface::{ControlSurfaceActuator, ControlSurfaceOrientation};

mod aero;
mod control_surface;
pub mod sensors;
mod thrust;
mod thrust_fx;
pub mod weapon;

use crate::environment::{CelestialBodyEnvironment, EnvironmentSet};
use crate::ship::aero::FuselageDrag;
use crate::ship::control_surface::ControlSurfacePosition;
use crate::ship::sensors::{FlightSensorData, LiftAndDragMeasurement, ThrustMeasurement};
use crate::ship::weapon::RotaryGun;
pub use aero::AeroSurface;
pub use thrust::Thrust;
use thrust_fx::{ThrusterEffectHandle, ThrusterParticle};

pub struct ShipPlugin;

/// Explicit simulation stages for ordering and parallelization
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum FlightSet {
    /// Gather sensor input to inform flight systems
    Sense,
    /// Process inputs, SAS damping, engine spooling, and actuator deflections
    Controls,
    /// Calculate and apply aero forces, drag, and engine thrust
    Forces,
}

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        // 1. Configure stage ordering in FixedUpdate
        app.configure_sets(
            FixedUpdate,
            (FlightSet::Sense, FlightSet::Controls, FlightSet::Forces)
                .after(EnvironmentSet)
                .chain(),
        )
        .add_systems(
            Startup,
            (thrust_fx::setup_thruster_effect, spawn_ship).chain(),
        )
        .add_systems(
            Update,
            (
                thrust_fx::update_thrust_particles,
                weapon::handle_gun_firing,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                (sensors::update_flight_sensors,).in_set(FlightSet::Sense),
                (
                    // Control surface chain (set -> update)
                    (
                        control_surface::set_control_surface_targets,
                        control_surface::update_control_surfaces,
                    )
                        .chain(),
                    // Throttle spooling runs in parallel with control surface chain
                    thrust::handle_throttle,
                )
                    .in_set(FlightSet::Controls),
                (aero::calculate_aerodynamic_forces, thrust::apply_thrust)
                    .in_set(FlightSet::Forces),
            ),
        );
    }
}

#[derive(Component, Clone, Default)]
pub struct Player;

#[derive(Component, Clone, Default)]
#[require(
    CelestialBodyEnvironment,
    FlightSensorData,
    RigidBody::Dynamic,
    CellCoord
)]
pub struct Ship;

pub fn spawn_ship(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    effect_handle: Res<ThrusterEffectHandle>,
    space: Single<Entity, With<BigSpace>>,
) {
    info!("Spawning F-16 spec fighter");
    let input_map = create_input_map();
    let asset_scale = 4.0;

    // Base wing aerodynamic profile (Cropped Delta + LERX vortex lift)
    let wing_aero = AeroSurface {
        area: 14.0, // 28 m² total main wing area
        lift_slope: 3.8,
        drag_0: 0.018,
        induced_drag_coeff: 0.14,
        stall_angle: f32::to_radians(24.0),
    };

    // All-moving horizontal taileron aero profile
    let tail_aero = AeroSurface {
        area: 3.0, // 6 m² total tailplane area
        lift_slope: 4.2,
        drag_0: 0.015,
        induced_drag_coeff: 0.12,
        stall_angle: f32::to_radians(28.0),
    };

    let vertical_fin_aero = AeroSurface {
        area: 4.5, // 4.5 m² vertical stabilizer
        lift_slope: 3.8,
        drag_0: 0.018,
        induced_drag_coeff: 0.15,
        stall_angle: f32::to_radians(22.0),
    };

    let fin_rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
    let initial_pos = Vec3::new(-1_000_000.0, 1500.0, 10.0);

    let ship_id =commands
        .spawn_scene(bsn! {
            Player
            Ship
            RotaryGun::default()
            Visibility::default()

            LiftAndDragMeasurement
            ThrustMeasurement

            template_value(LinearVelocity(Vec3::NEG_Z * 200.0))
            template_value(Position::from(initial_pos))
            Transform::from_translation(initial_pos)
            Collider::cuboid(10.0, 3.5, 15.0)
            Mass(11_000.0)
            FuselageDrag {
                forward_area: 0.09,
                side_area: 2.2,
                top_area: 3.8,
            }

            Children [
                // --- MAIN WINGS ---
                (
                    #LeftWing
                    template_value(wing_aero)
                    Mesh3d(asset_value(Cuboid::new(3.5, 0.1, 4.0)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.35, 0.4, 0.45)))
                    Transform::from_xyz(-2.8, 0.0, -0.2)
                )
                (
                    #RightWing
                    template_value(wing_aero)
                    Mesh3d(asset_value(Cuboid::new(3.5, 0.1, 4.0)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.35, 0.4, 0.45)))
                    Transform::from_xyz(2.8, 0.0, -0.2)
                ),

                // --- FLAPERONS / AILERONS ---
                (
                    #LeftAileron
                    template_value(AeroSurface { area: 2.5, ..wing_aero })
                    template_value(ControlSurfaceOrientation::Roll(ControlSurfacePosition::Left))
                    ControlSurfaceActuator {
                        max_angle: f32::to_radians(20.0),
                        speed: 10.0,
                    }
                    Mesh3d(asset_value(Cuboid::new(1.8, 0.12, 1.2)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.2, 0.8, 0.9)))
                    Transform::from_xyz(-4.5, 0.0, 0.8)
                )
                (
                    #RightAileron
                    template_value(AeroSurface { area: 2.5, ..wing_aero })
                    template_value(ControlSurfaceOrientation::Roll(ControlSurfacePosition::Right))
                    ControlSurfaceActuator {
                        max_angle: f32::to_radians(20.0),
                        speed: 10.0,
                    }
                    Mesh3d(asset_value(Cuboid::new(1.8, 0.12, 1.2)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.2, 0.8, 0.9)))
                    Transform::from_xyz(4.5, 0.0, 0.8)
                ),

                // --- ALL-MOVING HORIZONTAL TAILERONS ---
                (
                    #LeftTaileron
                    template_value(tail_aero)
                    template_value(ControlSurfaceOrientation::RollPitch(ControlSurfacePosition::Left))
                    ControlSurfaceActuator {
                        max_angle: f32::to_radians(25.0),
                        speed: 8.0,
                    }
                    Mesh3d(asset_value(Cuboid::new(2.2, 0.1, 1.8)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.95, 0.5, 0.1)))
                    Transform::from_xyz(-1.8, 0.2, 5.5)
                )
                (
                    #RightTaileron
                    template_value(tail_aero)
                    template_value(ControlSurfaceOrientation::RollPitch(ControlSurfacePosition::Right))
                    ControlSurfaceActuator {
                        max_angle: f32::to_radians(25.0),
                        speed: 8.0,
                    }
                    Mesh3d(asset_value(Cuboid::new(2.2, 0.1, 1.8)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.95, 0.5, 0.1)))
                    Transform::from_xyz(1.8, 0.2, 5.5)
                ),

                // --- VERTICAL STABILIZER & RUDDER ---
                (
                    #VerticalFin
                    template_value(vertical_fin_aero)
                    Mesh3d(asset_value(Cuboid::new(2.5, 0.1, 2.0)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.35, 0.4, 0.45)))
                    template_value(Transform::from_xyz(0.0, 1.6, 4.8).with_rotation(fin_rot))
                ),
                (
                    #Rudder
                    template_value(AeroSurface { area: 1.5, ..vertical_fin_aero })
                    template_value(ControlSurfaceOrientation::Yaw)
                    ControlSurfaceActuator {
                        max_angle: f32::to_radians(30.0),
                        speed: 7.0,
                        base_rotation: fin_rot,
                    }
                    Mesh3d(asset_value(Cuboid::new(1.8, 0.12, 1.0)))
                    MeshMaterial3d<StandardMaterial>(asset_value(Color::srgb(0.9, 0.2, 0.6)))
                    template_value(Transform::from_xyz(0.0, 1.6, 5.8).with_rotation(fin_rot))
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
                parent.spawn((
                    Thrust::new(1.0, 145_000. / 2., 0.55, 6.0),
                    Name::new("JetExhaust"),
                    ThrusterParticle,
                    ParticleEffect::new(effect_handle.0.clone()),
                    EffectProperties::default(),
                    Transform::from_xyz(-x_offset * 1.5, 0.0, 4.2),
                ));
            }
        }).id();

    commands.entity(*space).add_child(ship_id);
}
