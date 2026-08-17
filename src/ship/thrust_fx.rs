use crate::ship::{Ship, Thrust};
use avian3d::parry::glamx::{Vec3, Vec4};
use bevy::asset::{Assets, Handle};
use bevy::prelude::{Commands, Component, Query, Reflect, ResMut, Resource, With, default};
use bevy_hanabi::{Attribute, ColorOverLifetimeModifier, EffectAsset, EffectProperties, EffectSpawner, ExprWriter, SetAttributeModifier, SetPositionSphereModifier, ShapeDimension, SimulationSpace, SizeOverLifetimeModifier, SpawnerSettings};

#[derive(Resource)]
pub struct ThrusterEffectHandle(pub Handle<EffectAsset>);

#[derive(Component, Default, Clone, Reflect)]
pub struct ThrusterParticle;

pub fn setup_thruster_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    // 1. Color gradient over lifetime: bright core -> hot orange -> dark smoke -> transparent
    let mut color_gradient = bevy_hanabi::Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(0.6, 0.9, 1.0, 1.0)); // Blue-white core
    color_gradient.add_key(0.2, Vec4::new(1.0, 0.4, 0.1, 0.9)); // Orange flame
    color_gradient.add_key(0.6, Vec4::new(0.3, 0.3, 0.3, 0.4)); // Dark smoke
    color_gradient.add_key(1.0, Vec4::new(0.1, 0.1, 0.1, 0.0)); // Fade out

    // 2. 3D Size gradient
    let mut size_gradient = bevy_hanabi::Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(0.4));
    size_gradient.add_key(1.0, Vec3::splat(1.5));

    // 3. Use ExprWriter to build expressions with math operator overloading
    let writer = ExprWriter::new();

    let throttle_prop = writer.add_property("throttle", 0.0f32.into());
    let throttle_expr = writer.prop(throttle_prop);

    // Lifetime scales with throttle
    let base_lifetime = writer.lit(0.35f32);
    let lifetime = (base_lifetime * throttle_expr.clone()).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Position jitter in the nozzle
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.25f32).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Velocity scales with throttle (ejected backwards along local +Z)
    let base_exhaust_speed = writer.lit(Vec3::new(0.0, 0.0, 80.0));
    let init_vel = SetAttributeModifier::new(
        Attribute::VELOCITY,
        (base_exhaust_speed * throttle_expr).expr(),
    );

    let effect = EffectAsset::new(
        8192,
        SpawnerSettings::rate(400.0.into()),
        writer.finish(),
    )
    .with_name("jet_thruster")
    .with_simulation_space(SimulationSpace::Global)
    .init(init_pos)
    .init(init_lifetime)
    .init(init_vel)
    .render(ColorOverLifetimeModifier {
        gradient: color_gradient,
        ..default()
    })
    .render(SizeOverLifetimeModifier {
        gradient: size_gradient,
        screen_space_size: false,
    });

    commands.insert_resource(ThrusterEffectHandle(effects.add(effect)));
}

pub fn update_thrust_particles(
    ship_query: Query<&Thrust, With<Ship>>,
    mut particle_query: Query<(
        &mut EffectProperties,
        Option<&mut EffectSpawner>,
    ), With<ThrusterParticle>>,
) {
    let Ok(thrust) = ship_query.single() else {
        return;
    };
    let throttle = thrust.current_throttle;

    for (mut properties, maybe_spawner) in particle_query.iter_mut() {
        // 1. Pass the dynamic throttle property to the GPU compute shader
        properties.set("throttle", throttle.into());

        // 2. Adjust emission rate once Hanabi initializes the spawner
        if let Some(mut spawner) = maybe_spawner {
            if throttle > 0.01 {
                spawner.active = true;
                spawner.settings = SpawnerSettings::rate((throttle * 600.0).into());
            } else {
                spawner.active = false;
            }
        }
    }
}