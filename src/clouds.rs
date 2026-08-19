use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_hanabi::prelude::*;
use bevy_rand::prelude::*;
use rand::prelude::*;

pub struct CloudsPlugin;

impl Plugin for CloudsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_clouds);
    }
}

fn spawn_clouds(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut images: ResMut<Assets<Image>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    let puff_texture_handle = images.add(create_soft_cloud_puff_texture(128));

    // 1. Build distinct cloud archetypes with varied extents (Length, Height, Width)

    // Small, classic puffy cumulus
    let small_cloud = effects.add(create_cloud_effect_asset(
        "CumulusSmall",
        96,
        80.0,
        Vec3::new(300.0, 160.0, 300.0),
        190.0,
        0.28,
    ));

    // Long, flat, stretched-out cloud street / roll cloud (~3.5 km long)
    let roll_cloud = effects.add(create_cloud_effect_asset(
        "CloudStreet",
        256,
        220.0,
        Vec3::new(1_800.0, 130.0, 400.0),
        230.0,
        0.22,
    ));

    // Broad, expansive flat-bottomed cloud deck (~2.5 km x 2.0 km)
    let broad_deck = effects.add(create_cloud_effect_asset(
        "CumulusDeck",
        256,
        220.0,
        Vec3::new(1_300.0, 160.0, 1_000.0),
        250.0,
        0.20,
    ));

    // Tall, billowy convective cumulus tower
    let tower_cloud = effects.add(create_cloud_effect_asset(
        "CumulusTower",
        180,
        150.0,
        Vec3::new(450.0, 600.0, 450.0),
        260.0,
        0.24,
    ));

    let cloud_archetypes = [small_cloud, roll_cloud, broad_deck, tower_cloud];

    // 2. Procedurally scatter clouds across a ~35km x 35km airspace
    let num_clouds = 45;

    for i in 0..num_clouds {
        let x = rng.random_range(-17_000.0..17_000.0);
        let y = rng.random_range(850.0..2_400.0);
        let z = rng.random_range(-17_000.0..17_000.0);

        // Random yaw rotation to align elongated clouds in different directions
        let yaw = rng.random_range(0.0..std::f32::consts::TAU);
        let rotation = Quat::from_rotation_y(yaw);

        let archetype_idx = (rng.next_u32() as usize) % cloud_archetypes.len();
        let selected_effect = cloud_archetypes
            .choose(&mut rng)
            .expect("Cloud archetypes empty")
            .clone();

        // Spawn primary cloud formation
        commands.spawn((
            Name::new(format!("Cloud_{i}")),
            ParticleEffect::new(selected_effect),
            EffectMaterial {
                images: vec![puff_texture_handle.clone()],
            },
            Transform::from_translation(Vec3::new(x, y, z)).with_rotation(rotation),
        ));

        // For large decks and rolls, occasionally chain an offset companion for organic shape breaks
        if (archetype_idx == 1 || archetype_idx == 2) && rng.random_range(0.0..1.0) > 0.40 {
            let offset_local = Vec3::new(
                rng.random_range(-600.0..600.0),
                rng.random_range(-80.0..80.0),
                rng.random_range(-300.0..300.0),
            );
            let offset_world = rotation * offset_local;

            commands.spawn((
                Name::new(format!("Cloud_{i}_Companion")),
                ParticleEffect::new(cloud_archetypes[0].clone()),
                EffectMaterial {
                    images: vec![puff_texture_handle.clone()],
                },
                Transform::from_translation(Vec3::new(x, y, z) + offset_world)
                    .with_rotation(rotation),
            ));
        }
    }
}

/// Helper to create parametrized ellipsoidal cloud effect assets
fn create_cloud_effect_asset(
    name: &'static str,
    capacity: u32,
    puff_count: f32,
    extents: Vec3,
    puff_size: f32,
    opacity: f32,
) -> EffectAsset {
    let mut module = Module::default();
    module.add_texture_slot("cloud_puff");
    let texture_slot = module.lit(0u32);

    // 1. Scatter in a unit sphere volume
    let init_sphere = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(1.0),
        dimension: ShapeDimension::Volume,
    };

    // 2. Scale unit sphere by (X, Y, Z) extents to form custom 3D ellipsoids (long rolls, flat decks, towers)
    let pos_attr = module.attr(Attribute::POSITION);
    let scale_expr = module.lit(extents);
    let scaled_pos = module.mul(pos_attr, scale_expr);
    let set_scaled_pos = SetAttributeModifier::new(Attribute::POSITION, scaled_pos);

    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, module.lit(999_999.0));

    let render_size = SetSizeModifier {
        size: Vec3::splat(puff_size).into(),
    };

    let mut gradient = bevy_hanabi::Gradient::new();
    gradient.add_key(0.0, Vec4::new(1.0, 1.0, 1.0, opacity));
    gradient.add_key(1.0, Vec4::new(1.0, 1.0, 1.0, opacity));

    EffectAsset::new(capacity, SpawnerSettings::once(puff_count.into()), module)
        .with_name(name)
        // Disables position-velocity integration warnings for static particles
        .with_motion_integration(MotionIntegration::None)
        .init(init_sphere)
        .init(set_scaled_pos)
        .init(init_lifetime)
        .render(render_size)
        .render(ColorOverLifetimeModifier::new(gradient))
        .render(ParticleTextureModifier {
            texture_slot,
            sample_mapping: ImageSampleMapping::Modulate,
        })
        .render(OrientModifier {
            mode: OrientMode::FaceCameraPosition,
            rotation: None,
        })
}

/// Generates a smooth cosine-smoothed radial gradient (1.0 at center -> 0.0 at radius)
fn create_soft_cloud_puff_texture(size: u32) -> Image {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    let center = size as f32 / 2.0;
    let max_radius = center;

    for y in 0..size {
        for x in 0..size {
            let dx = (x as f32 + 0.5) - center;
            let dy = (y as f32 + 0.5) - center;
            let dist = (dx * dx + dy * dy).sqrt();

            let alpha = if dist < max_radius {
                let t = dist / max_radius;
                ((1.0 - t * t).max(0.0)).powi(2)
            } else {
                0.0
            };

            let a = (alpha * 255.0).clamp(0.0, 255.0) as u8;
            data.extend_from_slice(&[255, 255, 255, a]);
        }
    }

    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}
