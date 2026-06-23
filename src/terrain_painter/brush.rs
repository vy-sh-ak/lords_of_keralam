use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::terrain::{MapGenerator, TerrainSampler, chunk_span};
use crate::terrain_painter::{BrushConfig, TerrainTool};

pub fn get_affected_chunks(
    hit_pos: Vec3,
    radius: f32,
    map_generator: &MapGenerator,
) -> HashSet<IVec2> {
    let chunk_span_val = chunk_span(map_generator);
    let hit_chunk = IVec2::new(
        (hit_pos.x / chunk_span_val).round() as i32,
        (hit_pos.z / chunk_span_val).round() as i32,
    );
    let chunk_radius_span = (radius / chunk_span_val).ceil() as i32 + 1;

    let mut affected = HashSet::new();
    for cy in (hit_chunk.y - chunk_radius_span)..=(hit_chunk.y + chunk_radius_span) {
        for cx in (hit_chunk.x - chunk_radius_span)..=(hit_chunk.x + chunk_radius_span) {
            let coord = IVec2::new(cx, cy);
            let center_x = cx as f32 * chunk_span_val;
            let center_z = cy as f32 * chunk_span_val;
            let half_span = chunk_span_val / 2.0;
            if (hit_pos.x - center_x).abs() > half_span + radius
                || (hit_pos.z - center_z).abs() > half_span + radius
            {
                continue;
            }
            affected.insert(coord);
        }
    }
    affected
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
