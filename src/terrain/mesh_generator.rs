use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

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
    pub fn generate_terrain_mesh(height_map: &[Vec<f32>], height_multiplier: f32) -> MeshData {
        let height = height_map.len();
        let width = height_map.first().map_or(0, |row| row.len());

        let top_left_x = (width as f32 - 1.0) / -2.0;
        let top_left_z = (height as f32 - 1.0) / 2.0;

        let mut mesh_data = MeshData::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let vertex_pos = Vec3::new(
                    top_left_x + x as f32,
                    height_map[y][x] * height_multiplier,
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

                if x < width - 1 && y < height - 1 {
                    let vertex_index = (y * width + x) as u32;
                    let width_u32 = width as u32;

                    mesh_data.add_triangle(
                        vertex_index,
                        vertex_index + width_u32 + 1,
                        vertex_index + width_u32,
                    );
                    mesh_data.add_triangle(
                        vertex_index + width_u32 + 1,
                        vertex_index,
                        vertex_index + 1,
                    );
                }
            }
        }
        mesh_data
    }
}
