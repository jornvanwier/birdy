use crate::environment::air_density::create_atmosphere;
use avian3d::prelude::*;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::light::atmosphere::ScatteringMedium;
use bevy::mesh::{SphereKind, SphereMeshBuilder};
use bevy::prelude::*;
use big_space::prelude::*;
use big_space::world_query::CellTransformReadOnlyItem;
use crate::environment::gravity;

#[derive(Component, Default, Debug, Clone)]
pub struct ClosestBody(pub Option<Entity>);

#[derive(Component, Default, Debug, Clone)]
pub struct CelestialBody {
    pub radius: f32,
    /// Acceleration due to gravity at sea level (e.g., 9.81 for Earth, 3.72 for Mars)
    pub surface_gravity: f32,
}

impl CelestialBody {
    /// Calculates gravitational acceleration magnitude at distance `r` from body center.
    /// Uses Newtonian inverse-square falloff: g(r) = g_0 * (R / r)^2
    #[inline]
    pub fn gravity_at_distance(&self, distance: f32) -> f32 {
        if distance <= 0.0 {
            return self.surface_gravity;
        }
        // Clamping to radius prevents runaway gravity if clipping into ground
        let r = distance.max(self.radius);
        let ratio = self.radius / r;
        self.surface_gravity * (ratio * ratio)
    }
}

pub fn determine_closest_celestial_body(
    grid: Single<&Grid, With<BigSpace>>,
    mut locations: Query<(&mut ClosestBody, CellTransformReadOnly)>,
    celestial_bodies: Query<(Entity, CellTransformReadOnly), With<CelestialBody>>,
) {
    for (mut result, CellTransformReadOnlyItem { cell, transform }) in locations.iter_mut() {
        *result = ClosestBody(
            celestial_bodies
                .iter()
                .map(
                    |(
                        body_entity,
                        CellTransformReadOnlyItem {
                            cell: body_cell,
                            transform: body_transform,
                        },
                    )| {
                        let body_pos = grid.grid_position_double(body_cell, body_transform);
                        let pos = grid.grid_position_double(cell, transform);

                        let distance = pos.distance(body_pos);
                        (distance, body_entity)
                    },
                )
                .min_by(|(a_dist, _), (b_dist, _)| a_dist.total_cmp(b_dist))
                .map(|(_, closest_entity)| closest_entity),
        );
    }
}

pub fn setup_celestial_bodies(
    mut commands: Commands,
    space: Single<Entity, With<BigSpace>>,
    mut mediums: ResMut<Assets<ScatteringMedium>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.entity(*space).with_children(|parent| {
        // Atmosphere
        let earth_medium = mediums.add(ScatteringMedium::earth(512, 512));

        // 1/10 earth
        let body_radius = 636_000.;
        let body_transform = Transform::from_xyz(0.0, -body_radius, 0.0);
        let atmosphere_height = 70_000.;
        let sea_level_density = 1.225;
        let atmos_ground_albedo = Vec3::splat(0.3);

        let mut body_entity_commands = spawn_celestial_body(
            &mut meshes,
            &mut materials,
            body_radius,
            parent,
            body_transform,
        );

        insert_atmosphere(
            &mut body_entity_commands,
            body_radius,
            atmosphere_height,
            sea_level_density,
            atmos_ground_albedo,
            earth_medium,
        );
    });
}

fn spawn_celestial_body<'a>(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    body_radius: f32,
    parent: &'a mut RelatedSpawnerCommands<ChildOf>,
    planet_transform: Transform,
) -> EntityCommands<'a> {
    // Ground Plane
    let planet_mesh = Mesh::from(SphereMeshBuilder::new(
        body_radius,
        SphereKind::Ico { subdivisions: 20 },
    ));
    let planet_collider = Collider::trimesh_from_mesh(&planet_mesh)
        .expect("Failed to create trimesh collider from planet mesh");
    parent.spawn((
        Name::new("Planet"),
        CelestialBody {
            radius: body_radius,
            surface_gravity: gravity::STANDARD_G,
        },
        Mesh3d(meshes.add(planet_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.35, 0.15),
            perceptual_roughness: 0.9,
            ..default()
        })),
        CellCoord::default(),
        planet_transform,
        RigidBody::Static,
        planet_collider,
    ))
}

fn insert_atmosphere(
    body_entity_commands: &mut EntityCommands,
    body_radius: f32,
    atmosphere_height: f32,
    sea_level_density: f32,
    atmos_ground_albedo: Vec3,
    earth_medium: Handle<ScatteringMedium>,
) {
    let (render_atmosphere, atmosphere_properties) = create_atmosphere(
        body_radius,
        atmosphere_height,
        sea_level_density,
        atmos_ground_albedo,
        earth_medium,
    );
    body_entity_commands.insert((
        render_atmosphere,
        atmosphere_properties,
    ));
}