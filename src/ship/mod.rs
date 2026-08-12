use crate::input::create_input_map;
use crate::ship::throttle::Throttle;
use avian3d::prelude::*;
use bevy::app::{App, Plugin, Startup};
use bevy::asset::AssetServer;
use bevy::camera::Camera3d;
use bevy::prelude::*;

mod throttle;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ship).add_systems(
            Update,
            (
                throttle::handle_throttle.before(apply_flight_forces),
                apply_flight_forces,
            ),
        );
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FlightModel {
    pub max_thrust: f32,       // Engine thrust in Newtons (N)
    pub wing_area: f32,        // Wing surface area (m^2)
    pub lift_coefficient: f32, // Base lift coefficient (C_L)
    pub lift_slope: f32,       // Rate of C_L gain per radian of AoA (C_L_alpha)
    pub drag_coefficient: f32, // Base drag coefficient (C_D)

    /// Angular control authority for Pitch, Yaw, Roll (rad/s^2 at max input)
    pub control_authority: Vec3,

    /// Weathercocking resistance strength for Pitch (X) and Yaw (Y).
    /// Higher values keep the nose locked more tightly onto the velocity vector.
    pub pitch_stability: f32,
    pub yaw_stability: f32,

    /// Rotational damping factors (prevents endless spin/oscillations)
    pub angular_damping: Vec3,
}

impl Default for FlightModel {
    fn default() -> Self {
        Self {
            max_thrust: 35000.0,
            wing_area: 12.0,
            lift_coefficient: 0.6,
            lift_slope: 1.5,
            drag_coefficient: 0.08,
            control_authority: Vec3::new(4.5, 3.0, 7.0), // Pitch, Yaw, Roll
            pitch_stability: 10.0,
            yaw_stability: 12.0,
            angular_damping: Vec3::new(4.0, 4.0, 4.0),
        }
    }
}

fn apply_flight_forces(mut query: Query<(Forces, &Transform, &Throttle, &FlightModel)>) {
    let air_density = 1.225;

    for (mut forces, transform, throttle, flight_model) in query.iter_mut() {
        let thrust_magnitude = flight_model.max_thrust * throttle.current;
        let thrust_force = transform.forward() * thrust_magnitude;
        forces.apply_force(thrust_force);

        let velocity = forces.linear_velocity();
        let speed = velocity.length();

        if speed > 0.01 {
            let vel_dir = velocity.normalize();
            let dynamic_pressure = 0.5 * air_density * speed.powi(2);

            let drag_magnitude =
                dynamic_pressure * flight_model.wing_area * flight_model.drag_coefficient;
            let drag_force = -vel_dir * drag_magnitude;
            forces.apply_force(drag_force);

            let up = transform.up();
            let lift_dir = (*up - vel_dir * up.dot(vel_dir)).normalize_or_zero();

            let forward_dot_vel = transform.forward().dot(vel_dir).clamp(-1.0, 1.0);
            // Cap angle of attack at ~25 degrees (0.43 rad) to simulate wing stall
            let angle_of_attack = forward_dot_vel.acos().min(0.43);
            let effective_cl =
                flight_model.lift_coefficient + (angle_of_attack * flight_model.lift_slope);

            let lift_magnitude = dynamic_pressure * flight_model.wing_area * effective_cl;
            let lift_force = lift_dir * lift_magnitude;
            forces.apply_force(lift_force);
        }
    }
}

#[derive(Component)]
pub struct Ship;

pub fn spawn_ship(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning ship");
    let input_map = create_input_map();

    commands
        .spawn(input_map)
        .insert((
            Name::new("Player"),
            Ship,
            FlightModel::default(),
            Throttle::new(0., 0.5, 5.0),
            Transform::from_xyz(0., 0., 10.),
            RigidBody::Dynamic,
            Collider::cuboid(2.0, 0.75, 1.5),
            Mass(800.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                WorldAssetRoot(asset_server.load("game/craft_speederD.glb#Scene0")),
                Transform::from_xyz(-2., -0.4, -1.5),
            ));
            parent.spawn((Camera3d::default(), Transform::from_xyz(0.0, 0.5, 6.0)));
        });
}
