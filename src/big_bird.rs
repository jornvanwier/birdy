use avian3d::physics_transform::PhysicsTransformConfig;
use avian3d::prelude::*;
use bevy::prelude::*;
use big_space::prelude::*;

pub struct BigSpaceAvianSyncPlugin;

impl Plugin for BigSpaceAvianSyncPlugin {
    fn build(&self, app: &mut App) {
        // Disable Avian's default transform hooks
        app.insert_resource(PhysicsTransformConfig {
            propagate_before_physics: false,
            transform_to_position: false,
            position_to_transform: false,
            transform_to_collider_scale: true,
        })
        // Before physics runs: update Avian from newly spawned or kinematic transforms
        .add_systems(
            FixedPostUpdate,
            write_avian_pos.before(PhysicsSystems::Prepare),
        )
        // After physics runs: update big_space transforms from Avian simulation
        .add_systems(
            FixedPostUpdate,
            read_avian_pos.after(PhysicsSystems::StepSimulation),
        );
    }
}

/// big_space -> Avian (Write)
/// Updates Avian's Position/Rotation when an entity is newly spawned
/// or when Kinematic/Static bodies are moved in Bevy.
pub fn write_avian_pos(
    grid: Single<&Grid, With<BigSpace>>,
    mut query: Query<
        (
            &CellCoord,
            &Transform,
            &mut Position,
            &mut Rotation,
            Option<&RigidBody>,
        ),
        Or<(
            Added<RigidBody>,
            (With<RigidBody>, Changed<Transform>),
            (With<RigidBody>, Changed<CellCoord>),
        )>,
    >,
) {
    let edge_len = grid.cell_edge_length();

    for (cell, transform, mut pos, mut rot, rb) in query.iter_mut() {
        // Skip dynamic bodies that are currently being actively simulated
        // unless they were just added to the world
        let is_kinematic_or_static = rb.map_or(true, |r| !r.is_dynamic());
        if !is_kinematic_or_static {
            continue; // Do not overwrite actively simulated dynamic bodies
        }

        // Reconstruct continuous world position from CellCoord + local Transform
        let cell_offset = Vec3::new(
            cell.x as f32 * edge_len,
            cell.y as f32 * edge_len,
            cell.z as f32 * edge_len,
        );
        let world_position = cell_offset + transform.translation;

        pos.0 = world_position;
        rot.0 = transform.rotation;
    }
}

/// Avian -> big_space (Read)
/// Reads simulated dynamic physics from Avian and updates big_space (CellCoord + Transform).
pub fn read_avian_pos(
    grid: Single<&Grid, With<BigSpace>>,
    mut query: Query<
        (&Position, &Rotation, &mut CellCoord, &mut Transform),
        (With<RigidBody>, Without<FloatingOrigin>),
    >,
) {
    for (pos, rot, mut cell, mut transform) in query.iter_mut() {
        // Deconstruct continuous Avian position into CellCoord + local Transform offset
        let (new_cell, new_translation) = grid.imprecise_translation_to_grid(pos.0);

        *cell = new_cell;
        transform.translation = new_translation;
        transform.rotation = rot.0;
    }
}
