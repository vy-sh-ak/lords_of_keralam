use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::terrain::{MapGenerator, TerrainSampler, chunk_span};
use super::sculpt_map::{get_affected_chunks};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainTool {
    Raise,
    Lower,
    Flatten,
    Smooth,
}

#[derive(Resource)]
pub struct BrushConfig {
    pub active: bool,
    pub tool: TerrainTool,
    pub radius: f32,
    pub strength: f32,
    pub flatten_target: Option<f32>,
    pub flatten_sampling: bool,
}

impl Default for BrushConfig {
    fn default() -> Self {
        Self {
            active: false,
            tool: TerrainTool::Raise,
            radius: 5.0,
            strength: 1.0,
            flatten_target: None,
            flatten_sampling: false,
        }
    }
}

pub fn apply_brush(
    sculpt_chunks: &mut HashMap<IVec2, Vec<Vec<f32>>>,
    hit_pos: Vec3,
    brush_config: &BrushConfig,
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
    dt: f32,
    invert: bool,
) -> HashSet<IVec2> {
    let chunk_span_val = chunk_span(map_generator);
    let map_size = map_generator.map_chunk_size as usize + 2;
    let half = map_generator.map_chunk_size as f32 / 2.0;
    let radius = brush_config.radius;

    let mut affected_chunks = HashSet::new();
    let candidates = get_affected_chunks(hit_pos, radius, map_generator);

    for &coord in &candidates {
        let sculpt = sculpt_chunks.entry(coord).or_insert_with(|| {
            vec![vec![0.0_f32; map_size]; map_size]
        });

        let chunk_offset_x = coord.x as f32 * chunk_span_val;
        let chunk_offset_z = coord.y as f32 * chunk_span_val;

        let mut any_changed = false;

        let sign = if invert { -1.0 } else { 1.0 };

        for y in 0..sculpt.len() {
            for x in 0..sculpt[y].len() {
                let world_x = chunk_offset_x + x as f32 - half;
                let world_z = chunk_offset_z + half - y as f32;

                let dx = world_x - hit_pos.x;
                let dz = world_z - hit_pos.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist >= radius {
                    continue;
                }

                let t = (1.0 - (dist / radius)).powf(2.0);
                let influence = t * brush_config.strength * dt;

                match brush_config.tool {
                    TerrainTool::Raise => {
                        sculpt[y][x] += sign * influence;
                    }
                    TerrainTool::Lower => {
                        sculpt[y][x] -= sign * influence;
                    }
                    TerrainTool::Flatten => {
                        if let Some(target) = brush_config.flatten_target {
                            let noise = terrain_sampler.sample_noise(
                                map_generator,
                                world_x as f64,
                                world_z as f64,
                            );
                            let base = map_generator
                                .terrain_data
                                .height_curve
                                .sample(noise)
                                * map_generator.terrain_data.height_multiplier;
                            let target_offset = target - base;
                            sculpt[y][x] += (target_offset - sculpt[y][x]) * influence;
                        }
                    }
                    TerrainTool::Smooth => {
                        let mut sum = 0.0;
                        let mut count = 0;
                        for &(ny, nx) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                            let ny = y as i32 + ny;
                            let nx = x as i32 + nx;
                            if ny >= 0
                                && ny < sculpt.len() as i32
                                && nx >= 0
                                && nx < sculpt[0].len() as i32
                            {
                                sum += sculpt[ny as usize][nx as usize];
                                count += 1;
                            }
                        }
                        if count > 0 {
                            let avg = sum / count as f32;
                            sculpt[y][x] += (avg - sculpt[y][x]) * influence;
                        }
                    }
                }

                any_changed = true;
            }
        }

        if any_changed {
            affected_chunks.insert(coord);
        }
    }

    affected_chunks
}
