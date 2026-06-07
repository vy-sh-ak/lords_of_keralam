use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

use super::HeightCurve;

pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub triangles: Vec<u32>,
}

pub struct MeshGenerator;

impl MeshData {
    pub fn new(mesh_width: usize, mesh_height: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(mesh_width * mesh_height),
            uvs: Vec::with_capacity(mesh_width * mesh_height),
            triangles: Vec::with_capacity((mesh_width - 1) * (mesh_height - 1) * 6),
        }
    }

    pub fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        self.triangles.push(a);
        self.triangles.push(b);
        self.triangles.push(c);
    }

    pub fn create_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );

        let positions: Vec<[f32; 3]> = self.vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
        let uvs: Vec<[f32; 2]> = self.uvs.iter().map(|uv| [uv.x, uv.y]).collect();

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(self.triangles));

        mesh.compute_normals();
        mesh
    }
}

impl MeshGenerator {
    pub fn generate_terrain_mesh(
        height_map: &[Vec<f32>],
        height_multiplier: f32,
        height_curve: &HeightCurve,
        level_of_detail: u32,
    ) -> MeshData {
        let height = height_map.len();
        let width = height_map.first().map_or(0, |row| row.len());

        let top_left_x = (width as f32 - 1.0) / -2.0;
        let top_left_z = (height as f32 - 1.0) / 2.0;

        let mesh_simplification_increment = if level_of_detail == 0 {
            1
        } else {
            level_of_detail * 2
        };
        let vertices_per_row = ((width - 1) / mesh_simplification_increment as usize) + 1;
        let vertices_per_column = ((height - 1) / mesh_simplification_increment as usize) + 1;
        let mut mesh_data = MeshData::new(vertices_per_row, vertices_per_column);

        for (mesh_y, y) in (0..height)
            .step_by(mesh_simplification_increment as usize)
            .enumerate()
        {
            for (mesh_x, x) in (0..width)
                .step_by(mesh_simplification_increment as usize)
                .enumerate()
            {
                let curved_height = height_curve.sample(height_map[y][x]);
                let vertex_pos = Vec3::new(
                    top_left_x + x as f32,
                    curved_height * height_multiplier,
                    top_left_z - y as f32,
                );
                mesh_data.vertices.push(vertex_pos);

                let uv_coords = Vec2::new(
                    if width > 1 {
                        x as f32 / (width as f32 - 1.0)
                    } else {
                        0.0
                    },
                    if height > 1 {
                        y as f32 / (height as f32 - 1.0)
                    } else {
                        0.0
                    },
                );
                mesh_data.uvs.push(uv_coords);

                if mesh_x + 1 < vertices_per_row && mesh_y + 1 < vertices_per_column {
                    let vertex_index = (mesh_y * vertices_per_row + mesh_x) as u32;
                    let vertices_per_row_u32 = vertices_per_row as u32;

                    mesh_data.add_triangle(
                        vertex_index,
                        vertex_index + vertices_per_row_u32 + 1,
                        vertex_index + vertices_per_row_u32,
                    );
                    mesh_data.add_triangle(
                        vertex_index + vertices_per_row_u32 + 1,
                        vertex_index,
                        vertex_index + 1,
                    );
                }
            }
        }
        mesh_data
    }
}
