use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

use super::HeightCurve;

pub struct MeshGenerator;

impl MeshGenerator {
    pub fn generate_terrain_mesh(
        height_map: &[Vec<f32>],
        height_multiplier: f32,
        height_curve: &HeightCurve,
        level_of_detail: u32,
    ) -> MeshData {
        // let height = height_map.len();
        let bordered_size = height_map.first().map_or(0, |row| row.len());

        let mesh_simplification_increment = if level_of_detail == 0 {
            1
        } else {
            level_of_detail * 2
        };
        let inc_usize = mesh_simplification_increment as usize;
        let bordered_size_f = bordered_size as f32;
        let inc_f = mesh_simplification_increment as f32;

        let mesh_size_f = bordered_size_f - 2.0 * inc_f;
        let mesh_size_unsimplified_f = bordered_size_f - 2.0;

        let top_left_x = (mesh_size_unsimplified_f - 1.0) / -2.0;
        let top_left_z = (mesh_size_unsimplified_f - 1.0) / 2.0;

        let vertices_per_line = ((mesh_size_f - 1.0) / inc_f) as usize + 1;
        let mut mesh_data = MeshData::new(vertices_per_line);

        let mut vertex_indices_map: Vec<Vec<u32>> = vec![vec![0; bordered_size]; bordered_size];
        let mut border_vertex_index: i32 = -1;
        let mut mesh_vertex_index: u32 = 0;

        for y in (0..bordered_size).step_by(inc_usize) {
            for x in (0..bordered_size).step_by(inc_usize) {
                let is_border_vertex =
                    y == 0 || y == bordered_size - 1 || x == 0 || x == bordered_size - 1;
                if is_border_vertex {
                    vertex_indices_map[x][y] = border_vertex_index as u32;
                    border_vertex_index -= 1;
                } else {
                    vertex_indices_map[x][y] = mesh_vertex_index;
                    mesh_vertex_index += 1;
                }
            }
        }
        for y in (0..bordered_size).step_by(inc_usize) {
            for x in (0..bordered_size).step_by(inc_usize) {
                let vertex_index = vertex_indices_map[x][y] as i32;
                let x_f = x as f32;
                let y_f = y as f32;
                let percent: Vec2 =
                    Vec2::new((x_f - inc_f) / mesh_size_f, (y_f - inc_f) / mesh_size_f);
                let height: f32 = height_curve.sample(height_map[y][x]) * height_multiplier;
                let vertex_pos: Vec3 = Vec3::new(
                    top_left_x + percent.x * mesh_size_unsimplified_f,
                    height,
                    top_left_z - percent.y * mesh_size_unsimplified_f,
                );

                mesh_data.add_vertex(vertex_pos, percent, vertex_index);

                if x < bordered_size - 1 && y < bordered_size - 1 {
                    let a = vertex_indices_map[x][y];
                    let b = vertex_indices_map[x + inc_usize][y];
                    let c = vertex_indices_map[x][y + inc_usize];
                    let d = vertex_indices_map[x + inc_usize][y + inc_usize];

                    mesh_data.add_triangle(a, d, c);
                    mesh_data.add_triangle(d, a, b);
                }
            }
        }

        mesh_data
    }
}

#[derive(Default, Clone)]
pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub triangles: Vec<u32>,

    border_vertices: Vec<Vec3>,
    border_triangles: Vec<i32>,

    triangle_index: Option<usize>,
    border_triangle_index: Option<usize>,
}

impl MeshData {
    pub fn new(vertices_per_line: usize) -> Self {
        Self {
            vertices: vec![Vec3::ZERO; vertices_per_line * vertices_per_line],
            uvs: vec![Vec2::ZERO; vertices_per_line * vertices_per_line],
            triangles: vec![0; (vertices_per_line - 1) * (vertices_per_line - 1) * 6],

            border_vertices: vec![Vec3::ZERO; (vertices_per_line * 4) + 4],
            border_triangles: vec![0; vertices_per_line * 24],
            ..Default::default()
        }
    }

    pub fn add_vertex(&mut self, vertex: Vec3, uv: Vec2, vertex_index: i32) {
        if vertex_index < 0 {
            self.border_vertices[(-vertex_index - 1) as usize] = vertex;
        } else {
            let index = vertex_index as usize;
            self.vertices[index] = vertex;
            self.uvs[index] = uv;
        }
    }

    pub fn add_triangle(&mut self, a: u32, b: u32, c: u32) {
        let vertex_count_threshold = self.vertices.len() as u32;
        let is_border_triangle = a >= vertex_count_threshold
            || b >= vertex_count_threshold
            || c >= vertex_count_threshold;

        if is_border_triangle {
            let idx = self.border_triangle_index.unwrap_or(0);
            self.border_triangles[idx] = a as i32;
            self.border_triangles[idx + 1] = b as i32;
            self.border_triangles[idx + 2] = c as i32;
            self.border_triangle_index = Some(idx + 3);
        } else {
            let idx = self.triangle_index.unwrap_or(0);
            self.triangles[idx] = a;
            self.triangles[idx + 1] = b;
            self.triangles[idx + 2] = c;
            self.triangle_index = Some(idx + 3);
        }
    }
    fn calculate_normals(&self) -> Vec<Vec3> {
        let mut vertex_normals = vec![Vec3::ZERO; self.vertices.len()];
        let triangle_count = self.triangles.len() / 3;
        for i in 0..triangle_count {
            let normal_triangle_index = i * 3;
            let vertex_index_a = self.triangles[normal_triangle_index] as usize;
            let vertex_index_b = self.triangles[normal_triangle_index + 1] as usize;
            let vertex_index_c = self.triangles[normal_triangle_index + 2] as usize;

            let triangle_normal =
                self.surface_normal_from_indices(vertex_index_a, vertex_index_b, vertex_index_c);
            vertex_normals[vertex_index_a] += triangle_normal;
            vertex_normals[vertex_index_b] += triangle_normal;
            vertex_normals[vertex_index_c] += triangle_normal;
        }
        let border_triangle_count = self.border_triangles.len() / 3;
        for i in 0..border_triangle_count {
            let normal_triangle_index = i * 3;
            let vertex_index_a = self.border_triangles[normal_triangle_index];
            let vertex_index_b = self.border_triangles[normal_triangle_index + 1];
            let vertex_index_c = self.border_triangles[normal_triangle_index + 2];

            let triangle_normal: Vec3 = self.surface_normal_from_indices(
                vertex_index_a as usize,
                vertex_index_b as usize,
                vertex_index_c as usize,
            );

            if vertex_index_a >= 0 {
                vertex_normals[vertex_index_a as usize] += triangle_normal;
            }
            if vertex_index_b >= 0 {
                vertex_normals[vertex_index_b as usize] += triangle_normal;
            }
            if vertex_index_c >= 0 {
                vertex_normals[vertex_index_c as usize] += triangle_normal;
            }
        }
        for normal in vertex_normals.iter_mut() {
            if *normal != Vec3::ZERO {
                *normal = normal.normalize();
            }
        }
        vertex_normals
    }

    fn surface_normal_from_indices(&self, index_a: usize, index_b: usize, index_c: usize) -> Vec3 {
        let point_a = if (index_a as i32) < 0 {
            self.border_vertices[(-(index_a as i32) - 1) as usize]
        } else {
            self.vertices[index_a]
        };
        let point_b = if (index_b as i32) < 0 {
            self.border_vertices[(-(index_b as i32) - 1) as usize]
        } else {
            self.vertices[index_b]
        };
        let point_c = if (index_c as i32) < 0 {
            self.border_vertices[(-(index_c as i32) - 1) as usize]
        } else {
            self.vertices[index_c]
        };

        let side_ab = point_b - point_a;
        let side_ac = point_c - point_a;
        side_ab.cross(side_ac).normalize()
    }

    pub fn create_mesh(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );

        let positions: Vec<[f32; 3]> = self.vertices.iter().map(|v| [v.x, v.y, v.z]).collect();
        let uvs: Vec<[f32; 2]> = self.uvs.iter().map(|uv| [uv.x, uv.y]).collect();
        let normals = self.calculate_normals();

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(self.triangles));
        // mesh.compute_normals();
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh
    }
}
