use crate::terrain::{FallOffGenerator, MapData, MapGenerator, TerrainSampler};

use bevy::prelude::*;

pub(crate) fn chunk_span(map_generator: &MapGenerator) -> f32 {
    map_generator.map_chunk_size.saturating_sub(1).max(1) as f32
}

pub(crate) fn ensure_falloff_map(map_generator: &mut MapGenerator) {
    if map_generator.terrain_data.use_falloff_map && map_generator.falloff_map.is_empty() {
        let map_size = map_generator.map_chunk_size as usize + 2;
        map_generator.set_falloff_map(FallOffGenerator::generate_fall_off_map(map_size));
    }
}

pub(crate) fn generate_map_data(
    terrain_sampler: &TerrainSampler,
    coord: IVec2,
    map_generator: &MapGenerator,
) -> MapData {
    let mut noise_map = generate_noise_map_for_chunk(terrain_sampler, coord, map_generator);

    if map_generator.terrain_data.use_falloff_map {
        for y in 0..noise_map.len() {
            for x in 0..noise_map[y].len() {
                noise_map[y][x] =
                    (noise_map[y][x] - map_generator.falloff_map[y][x]).clamp(0.0, 1.0);
            }
        }
    }

    MapData::new(noise_map)
}

pub(crate) fn generate_noise_map_for_chunk(
    terrain_sampler: &TerrainSampler,
    chunk_coord: IVec2,
    map_generator: &MapGenerator,
) -> Vec<Vec<f32>> {
    let width = map_generator.map_chunk_size as usize + 2;
    let height = map_generator.map_chunk_size as usize + 2;
    let half_width = map_generator.map_chunk_size as f64 / 2.0;
    let half_height = map_generator.map_chunk_size as f64 / 2.0;
    let chunk_span = map_generator.map_chunk_size.saturating_sub(1) as f64;
    let chunk_offset_x = chunk_coord.x as f64 * chunk_span;
    let chunk_offset_y = chunk_coord.y as f64 * chunk_span;

    let mut noise_map = vec![vec![0.0_f32; width]; height];

    for y in 0..height {
        for x in 0..width {
            let world_x = chunk_offset_x + x as f64 - half_width;
            let world_z = chunk_offset_y - y as f64 + half_height;
            noise_map[y][x] = terrain_sampler.sample_noise(map_generator, world_x, world_z);
        }
    }

    noise_map
}