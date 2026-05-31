use bevy::{
    asset::RenderAssetUsages,
    mesh::Indices,
    prelude::*,
    render::render_resource::PrimitiveTopology,
};

use crate::terrain::{MainWorld, TERRAIN_MAX_X, TERRAIN_MAX_Z, TERRAIN_MIN_X, TERRAIN_MIN_Z};

pub struct SmoothTerrainPlugin;

const CELL_SIZE: f32 = 2.0;

impl Plugin for SmoothTerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_smooth_terrain);
    }
}

fn spawn_smooth_terrain(
    mut commands: Commands,
    world: Res<MainWorld>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = build_heightfield_mesh(&world, CELL_SIZE);
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.34, 0.50, 0.28),
        perceptual_roughness: 1.0,
        ..default()
    });

    commands.spawn((
        Name::new("SmoothTerrain"),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::default(),
    ));
}

fn build_heightfield_mesh(world: &MainWorld, cell_size: f32) -> Mesh {
    let min_x = TERRAIN_MIN_X as f32;
    let max_x = TERRAIN_MAX_X as f32;
    let min_z = TERRAIN_MIN_Z as f32;
    let max_z = TERRAIN_MAX_Z as f32;

    let verts_x = ((max_x - min_x) / cell_size).round() as usize + 1;
    let verts_z = ((max_z - min_z) / cell_size).round() as usize + 1;

    let mut positions = Vec::with_capacity(verts_x * verts_z);
    let mut normals = Vec::with_capacity(verts_x * verts_z);
    let mut uvs = Vec::with_capacity(verts_x * verts_z);
    let mut indices = Vec::with_capacity((verts_x - 1) * (verts_z - 1) * 6);

    for z_i in 0..verts_z {
        for x_i in 0..verts_x {
            let x = min_x + x_i as f32 * cell_size;
            let z = min_z + z_i as f32 * cell_size;
            let y = world.sample_height(x, z);

            let h_l = world.sample_height(x - cell_size, z);
            let h_r = world.sample_height(x + cell_size, z);
            let h_d = world.sample_height(x, z - cell_size);
            let h_u = world.sample_height(x, z + cell_size);

            let normal = Vec3::new(h_l - h_r, 2.0 * cell_size, h_d - h_u).normalize();

            positions.push([x, y, z]);
            normals.push(normal.to_array());
            uvs.push([
                x_i as f32 / (verts_x - 1) as f32,
                z_i as f32 / (verts_z - 1) as f32,
            ]);
        }
    }

    for z_i in 0..verts_z - 1 {
        for x_i in 0..verts_x - 1 {
            let i0 = (z_i * verts_x + x_i) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + verts_x as u32;
            let i3 = i2 + 1;

            indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}