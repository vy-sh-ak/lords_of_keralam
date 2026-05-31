use std::sync::Arc;

use bevy::{diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin}, platform::collections::HashMap, prelude::*};
use bevy_voxel_world::{custom_meshing::{CHUNK_SIZE_I, CHUNK_SIZE_U}, prelude::*};
use noise::{HybridMulti, NoiseFn, Perlin};

pub const CAMERA_VEC3: Vec3 = Vec3::new(-200.0, 180.0, -200.0);
pub const TERRAIN_SIZE_X: i32 = 1000;
pub const TERRAIN_SIZE_Z: i32 = 1000;

const TERRAIN_HALF_SIZE_X: i32 = TERRAIN_SIZE_X / 2;
const TERRAIN_HALF_SIZE_Z: i32 = TERRAIN_SIZE_Z / 2;
const TERRAIN_MIN_X: i32 = -TERRAIN_HALF_SIZE_X;
const TERRAIN_MAX_X: i32 = TERRAIN_MIN_X + TERRAIN_SIZE_X;
const TERRAIN_MIN_Z: i32 = -TERRAIN_HALF_SIZE_Z;
const TERRAIN_MAX_Z: i32 = TERRAIN_MIN_Z + TERRAIN_SIZE_Z;

fn chunk_is_outside_terrain_bounds(chunk_min: IVec3, chunk_max: IVec3) -> bool {
    chunk_max.x <= TERRAIN_MIN_X
        || chunk_min.x >= TERRAIN_MAX_X
        || chunk_max.z <= TERRAIN_MIN_Z
        || chunk_min.z >= TERRAIN_MAX_Z
}

fn voxel_is_outside_terrain_bounds(pos: IVec3) -> bool {
    pos.x < TERRAIN_MIN_X
        || pos.x >= TERRAIN_MAX_X
        || pos.z < TERRAIN_MIN_Z
        || pos.z >= TERRAIN_MAX_Z
}

#[derive(Component)]
pub struct FpsText;

#[derive(Resource, Clone)]
pub struct MainWorld {
    noise: Arc<HybridMulti<Perlin>>,
}

impl Default for MainWorld {
    fn default() -> Self {
        let mut noise = HybridMulti::<Perlin>::new(1234);
        noise.octaves = 5;
        noise.frequency = 1.1;
        noise.lacunarity = 2.8;
        noise.persistence = 0.4;

        Self {
            noise: Arc::new(noise),
        }
    }
}

impl VoxelWorldConfig for MainWorld {
    type MaterialIndex = u8;
    type ChunkUserBundle = ();

    fn spawning_distance(&self) -> u32 {
        // 25
        48
    }

    fn min_despawn_distance(&self) -> u32 {
        2
    }

    fn voxel_lookup_delegate(&self) -> VoxelLookupDelegate<Self::MaterialIndex> {
        let chunk_noise = Arc::clone(&self.noise);
        Box::new(move |chunk_pos, lod_level, _previous| {
            if chunk_pos.y < -1 {
                return Box::new(|_, _| WorldVoxel::Solid(3));
            }
            if chunk_pos.y > 4 {
                return Box::new(|_, _| WorldVoxel::Air);
            }

            let chunk_min = chunk_pos * CHUNK_SIZE_I;
            let chunk_max = chunk_min + IVec3::splat(CHUNK_SIZE_I);

            if chunk_is_outside_terrain_bounds(chunk_min, chunk_max) {
                return Box::new(|_, _| WorldVoxel::Air);
            }

            let skirt_enabled = lod_level == 2;

            let noise = Arc::clone(&chunk_noise);

            let mut cache = HashMap::<(i32, i32), f64>::new();

            Box::new(move |pos: IVec3, _previous_voxel| {
                if skirt_enabled {
                    let outside = pos.x < chunk_min.x
                        || pos.x >= chunk_max.x
                        || pos.y < chunk_min.y
                        || pos.y >= chunk_max.y
                        || pos.z < chunk_min.z
                        || pos.z >= chunk_max.z;
                    if outside {
                        return WorldVoxel::Unset;
                    }
                }

                if voxel_is_outside_terrain_bounds(pos) {
                    return WorldVoxel::Air;
                }

                // Sea level
                if pos.y < 0 {
                    return WorldVoxel::Solid(3);
                }

                let [x, y, z] = pos.as_dvec3().to_array();

                let is_ground = y < match cache.get(&(pos.x, pos.z)) {
                    Some(sample) => *sample,
                    None => {
                        let sample = noise.get([x / 1000.0, z / 1000.0]) * 50.0;
                        cache.insert((pos.x, pos.z), sample);
                        sample
                    }
                };

                if is_ground {
                    WorldVoxel::Solid(0)
                } else {
                    WorldVoxel::Air
                }
            })
        })
    }

    fn texture_index_mapper(&self) -> Arc<dyn Fn(Self::MaterialIndex) -> [u32; 3] + Send + Sync> {
        Arc::new(|mat| match mat {
            0 => [0, 0, 0],
            1 => [1, 1, 1],
            2 => [2, 2, 2],
            3 => [3, 3, 3],
            _ => [0, 0, 0],
        })
    }

    fn chunk_data_shape(&self, lod_level: LodLevel) -> UVec3 {
        padded_chunk_shape_uniform(CHUNK_SIZE_U / lod_level.max(1) as u32)
    }

    fn chunk_meshing_shape(&self, lod_level: LodLevel) -> UVec3 {
        padded_chunk_shape_uniform(CHUNK_SIZE_U / lod_level.max(1) as u32)
    }

    fn chunk_lod(
        &self,
        chunk_position: IVec3,
        _previous_lod: Option<LodLevel>,
        camera_position: Vec3,
    ) -> LodLevel
    {
        let camera_chunk = (camera_position / CHUNK_SIZE_U as f32).floor();
        let distance = chunk_position.as_vec3().distance(camera_chunk);

        if distance < 10.0 {
            1
        } else if distance <  25.0 {
            2
        } else if distance < 50.0 {
            4
        } else if distance < 85.0 {
            8
        } else if distance < 125.0 {
            16
        } else {
            32
        }
    }

    fn attach_chunks_to_root(&self) -> bool {
        false
    }
}

pub fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut text_query: Query<&mut Text, With<FpsText>>,
) {
    let Some(mut text) = text_query.iter_mut().next() else {
        return;
    };

    let fps_diag = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS);
    let fps_current = fps_diag.and_then(|d| d.value());
    let fps_avg = fps_diag.and_then(|d| d.average());
    let frame_ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.average())
        .map(|seconds| seconds * 1000.0);

    let line_one = match (fps_current, fps_avg) {
        (Some(current), Some(avg)) => {
            format!("FPS: {:>5.1} (avg {:>5.1})\n", current, avg)
        }
        (Some(current), None) => format!("FPS: {:>5.1} (--)\n", current),
        _ => "FPS: -- (--)\n".to_string(),
    };

    let line_two = match frame_ms {
        Some(ms) => format!("Frame: {:>5.2} ms", ms),
        None => "Frame: -- ms".to_string(),
    };

    text.0 = format!("{line_one}{line_two}");
}