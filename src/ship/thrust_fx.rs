use crate::ship::Thrust;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

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

    // 3. Use ExprWriter to build expressions with dynamic properties
    let writer = ExprWriter::new();

    let throttle_prop = writer.add_property("throttle", 0.0f32.into());
    let throttle_expr = writer.prop(throttle_prop);

    // Dynamic world-space exhaust velocity property
    let exhaust_vel_prop = writer.add_property("exhaust_velocity", Vec3::ZERO.into());
    let exhaust_vel_expr = writer.prop(exhaust_vel_prop);

    // Lifetime scales with throttle
    let base_lifetime = writer.lit(0.35f32);
    let lifetime = (base_lifetime * throttle_expr).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Position jitter in the nozzle
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.25f32).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Initial velocity uses the computed world-space exhaust velocity
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, exhaust_vel_expr.expr());

    let rotation = (writer.rand(ScalarType::Float) * writer.lit(std::f32::consts::TAU)).expr();
    let init_rotation = SetAttributeModifier::new(Attribute::F32_0, rotation);
    let rotation_attr = writer.attr(Attribute::F32_0).expr();

    let effect = EffectAsset::new(8192, SpawnerSettings::rate(400.0.into()), writer.finish())
        .with_name("jet_thruster")
        .with_simulation_space(SimulationSpace::Global)
        .init(init_pos)
        .init(init_lifetime)
        .init(init_rotation)
        .init(init_vel)
        .render(OrientModifier {
            mode: OrientMode::FaceCameraPosition,
            rotation: Some(rotation_attr),
        })
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
    ship_query: Query<Option<&LinearVelocity>>,
    mut particle_query: Query<
        (
            &Thrust,
            &GlobalTransform,
            &ChildOf,
            &mut EffectProperties,
            &mut EffectSpawner,
        ),
        With<ThrusterParticle>,
    >,
) {
    for (thrust, thruster_transform, child_of, mut properties, mut spawner) in
        particle_query.iter_mut()
    {
        // Get linear velocity from parent ship entity
        let ship_vel = ship_query
            .get(child_of.parent())
            .ok()
            .flatten()
            .map(|v| v.0)
            .unwrap_or(Vec3::ZERO);

        let throttle = thrust.current_throttle;

        // Backward exhaust vector in world space (+Z local)
        let back_dir = *thruster_transform.back();
        let exhaust_vel = ship_vel + back_dir * (80.0 * throttle);

        // 1. Pass properties to the GPU compute shader
        properties.set("throttle", throttle.into());
        properties.set("exhaust_velocity", exhaust_vel.into());

        // 2. Adjust emission rate based on throttle
        if throttle > 0.01 {
            spawner.active = true;
            spawner.settings = SpawnerSettings::rate((throttle * 600.0).into());
        } else {
            spawner.active = false;
        }
    }
}
