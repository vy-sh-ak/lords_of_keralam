use crate::terrain::MapGenerator;

use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{RngExt, SeedableRng, rngs::StdRng};

#[derive(Resource)]
pub struct TerrainSampler {
    perlin: Perlin,
}
impl Default for TerrainSampler {
    fn default() -> Self {
        Self {
            perlin: Perlin::new(0),
        }
    }
}
impl TerrainSampler {
    pub fn sample_noise(&self, map_generator: &MapGenerator, world_x: f64, world_z: f64) -> f32 {
        let octaves = map_generator.noise_data.octaves.max(1) as usize;

        let octave_offsets = self.build_octave_offsets(&map_generator, octaves);

        let scale = map_generator.noise_data.scale.max(0.0001);

        let max_possible_height = self.max_possible_noise_height(&map_generator, octaves);

        let mut amplitude = 1.0;
        let mut frequency = map_generator.noise_data.frequency.max(0.0001);

        let mut noise_height = 0.0;

        for &(offset_x, offset_y) in &octave_offsets {
            let sample_x = (world_x / scale) * frequency + offset_x;

            let sample_y = (world_z / scale) * frequency + offset_y;

            let perlin_value = self.perlin.get([sample_x, sample_y]);

            noise_height += perlin_value * amplitude;

            amplitude *= map_generator.noise_data.persistence;
            frequency *= map_generator.noise_data.lacunarity;
        }

        if max_possible_height <= f64::EPSILON {
            return 0.0;
        }

        (((noise_height + max_possible_height) / (max_possible_height * 2.0)).clamp(0.0, 1.0))
            as f32
    }

    pub fn sample_height(&self, map_generator: &MapGenerator, world_x: f32, world_z: f32) -> f32 {
        let noise = self.sample_noise(map_generator, world_x as f64, world_z as f64);

        let curved_height = map_generator.terrain_data.height_curve.sample(noise);

        curved_height * map_generator.terrain_data.height_multiplier
    }
    fn build_octave_offsets(&self, map_generator: &MapGenerator, octaves: usize) -> Vec<(f64, f64)> {
        let mut rng = StdRng::seed_from_u64(map_generator.noise_data.seed as u64);

        (0..octaves)
            .map(|_| {
                (
                    rng.random_range(-100_000.0..100_000.0) + map_generator.noise_data.offset_x,
                    rng.random_range(-100_000.0..100_000.0) + map_generator.noise_data.offset_y,
                )
            })
            .collect()
    }
    fn max_possible_noise_height(&self, map_generator: &MapGenerator, octaves: usize) -> f64 {
        let mut amplitude = 1.0;
        let mut max_possible_height = 0.0;

        for _ in 0..octaves {
            max_possible_height += amplitude;
            amplitude *= map_generator.noise_data.persistence;
        }

        max_possible_height
    }
}
