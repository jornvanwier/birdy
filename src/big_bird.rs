use avian3d::physics_transform::PhysicsTransformConfig;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use big_space::prelude::*;

pub struct BigSpaceAvianSyncPlugin;

impl Plugin for BigSpaceAvianSyncPlugin {
    fn build(&self, app: &mut App) {
        // Disable Avian's default transform sync to manage it with big_space
        app.insert_resource(PhysicsTransformConfig {
            propagate_before_physics: false,
            transform_to_position: false,
            position_to_transform: false,
            transform_to_collider_scale: true,
        })
        // 1. Before physics runs: align Avian Position with big_space relative to the current origin
        .add_systems(
            FixedPostUpdate,
            write_avian_pos.before(PhysicsSystems::Prepare),
        )
        // 2. After physics runs: write simulated positions back to big_space
        .add_systems(
            FixedPostUpdate,
            read_avian_pos.after(PhysicsSystems::StepSimulation),
        );
    }
}

/// big_space -> Avian (Write)
/// Syncs entities into Avian's local space relative to the FloatingOrigin cell.
pub fn write_avian_pos(
    grid: Single<&Grid, With<BigSpace>>,
    origin_query: Single<&CellCoord, With<FloatingOrigin>>,
    mut query: Query<(&CellCoord, &Transform, &mut Position, &mut Rotation)>,
) {
    let edge_len = grid.cell_edge_length();
    let origin_cell = *origin_query;

    for (cell, transform, mut pos, mut rot) in query.iter_mut() {
        // Compute continuous position relative to the FloatingOrigin cell
        let dx = (cell.x - origin_cell.x) as f32;
        let dy = (cell.y - origin_cell.y) as f32;
        let dz = (cell.z - origin_cell.z) as f32;
        let relative_cell_offset = Vec3::new(dx * edge_len, dy * edge_len, dz * edge_len);
        let target_pos = relative_cell_offset + transform.translation;

        // Kinematic/static bodies (or newly spawned dynamic bodies) always sync from Transform.
        // For actively simulated dynamic bodies, setting pos.0 here re-centers them if the
        // origin cell hopped between frames without resetting their physics velocities.
        pos.0 = target_pos;
        rot.0 = transform.rotation;
    }
}

/// Avian -> big_space (Read)
/// Reads simulated dynamic physics and updates big_space (CellCoord + Transform).
pub fn read_avian_pos(
    grid: Single<&Grid, With<BigSpace>>,
    origin_query: Single<&CellCoord, (With<FloatingOrigin>, Without<RigidBody>)>,
    mut query: Query<(&Position, &Rotation, &mut CellCoord, &mut Transform), With<RigidBody>>,
) {
    let origin_cell = *origin_query;

    for (pos, rot, mut cell, mut transform) in query.iter_mut() {
        // pos.0 is relative to origin_cell: decompose it into (cell_delta, local_translation)
        let (delta_cell, new_translation) = grid.imprecise_translation_to_grid(pos.0);

        *cell = CellCoord {
            x: origin_cell.x + delta_cell.x,
            y: origin_cell.y + delta_cell.y,
            z: origin_cell.z + delta_cell.z,
        };
        transform.translation = new_translation;
        transform.rotation = rot.0;
    }
}

// -----------------------------------------------------------------------------
// Hanabi Particle Sync
// -----------------------------------------------------------------------------

#[derive(Resource, Default)]
struct LastHanabiOriginCell(Option<CellCoord>);

pub struct BigSpaceHanabiSyncPlugin;

impl Plugin for BigSpaceHanabiSyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastHanabiOriginCell>()
            .add_systems(PostUpdate, sync_hanabi_floating_origin);
    }
}

fn sync_hanabi_floating_origin(
    grid: Single<&Grid, With<BigSpace>>,
    origin_query: Single<&CellCoord, With<FloatingOrigin>>,
    mut last_cell: ResMut<LastHanabiOriginCell>,
    mut effect_query: Query<&mut EffectProperties>,
) {
    let current_cell = *origin_query;
    let edge_len = grid.cell_edge_length();

    let origin_delta = if let Some(prev) = last_cell.0 {
        if prev != *current_cell {
            let dx = (current_cell.x - prev.x) as f32;
            let dy = (current_cell.y - prev.y) as f32;
            let dz = (current_cell.z - prev.z) as f32;

            -Vec3::new(dx * edge_len, dy * edge_len, dz * edge_len)
        } else {
            Vec3::ZERO
        }
    } else {
        Vec3::ZERO
    };

    last_cell.0 = Some(*current_cell);

    if origin_delta != Vec3::ZERO {
        for mut properties in effect_query.iter_mut() {
            properties.set("origin_delta", origin_delta.into());
        }
    }
}
