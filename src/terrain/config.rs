use super::HeightCurve;
use crate::terrain::data::{TextureLayerConfig, NoiseData, TerrainData, TextureData};
use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Reflect, InspectorOptions, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[reflect(InspectorOptions)]
pub enum DrawMode {
    NoiseMap,
    Mesh,
    EndlessTerrain,
    FallOffMap,
}
impl DrawMode {
    pub const ALL: [Self; 4] = [
        Self::NoiseMap,
        Self::Mesh,
        Self::EndlessTerrain,
        Self::FallOffMap,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::NoiseMap => "Noise Map",
            Self::Mesh => "Mesh",
            Self::EndlessTerrain => "Endless Terrain",
            Self::FallOffMap => "Fall-Off Map",
        }
    }
}

impl Default for DrawMode {
    fn default() -> Self {
        Self::NoiseMap
    }
}

#[derive(Reflect, InspectorOptions, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[reflect(InspectorOptions)]
pub struct EndlessTerrainLodBand {
    #[inspector(min = 0, max = 8)]
    pub level_of_detail: u32,
    #[inspector(min = 0.0, max = 10000.0)]
    pub visible_distance: f32,
}

impl EndlessTerrainLodBand {
    fn new(level_of_detail: u32, visible_distance: f32) -> Self {
        Self {
            level_of_detail,
            visible_distance,
        }
    }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[reflect(Resource, InspectorOptions)]
pub struct MapGenerator {
    #[reflect(ignore)]
    pub map_chunk_size: u32,
    // TODO: falloff_map is skipped from serialization and regenerated on load via ensure_falloff_map().
    // If the falloff generation parameters change, old saved configs won't have matching data.
    #[serde(skip)]
    #[reflect(ignore)]
    pub falloff_map: Vec<Vec<f32>>,

    #[inspector(min = 0, max = 3)]
    pub level_of_detail: u32,

    pub draw_mode: DrawMode,
    #[serde(default)]
    pub show_uv_wireframe: bool,
    #[serde(default = "default_endless_lod_bands")]
    pub endless_lod_bands: Vec<EndlessTerrainLodBand>,

    pub noise_data: NoiseData,
    pub terrain_data: TerrainData,
    pub texture_data: TextureData,
}

impl Default for MapGenerator {
    fn default() -> Self {
        Self {
            map_chunk_size: 241,
            level_of_detail: 0,
            draw_mode: DrawMode::default(),
            show_uv_wireframe: false,
            endless_lod_bands: default_endless_lod_bands(),
            falloff_map: vec![],
            noise_data: NoiseData {
                frequency: 0.3,
                scale: 10.0,
                octaves: 4,
                lacunarity: 2.0,
                persistence: 0.5,
                offset_x: 0.0,
                offset_y: 0.0,
                seed: 0,
            },
            terrain_data: TerrainData {
                use_falloff_map: false,
                height_multiplier: 20.0,
                height_curve: HeightCurve::default(),
            },
            texture_data: TextureData {
                min_height: -10.0,
                max_height: 40.0,
                layers: vec![
                    TextureLayerConfig {
                        texture_path: "textures/water.png".to_string(),
                        start_height: 0.2,
                        blend_strength: 0.1,
                        tint_strength: 0.0,
                        texture_scale: 5.0,
                        tint: LinearRgba::new(0.0, 0.0, 0.8, 1.0),
                    },
                    TextureLayerConfig {
                        texture_path: "textures/sand.png".to_string(),
                        start_height: 0.3,
                        blend_strength: 0.1,
                        tint_strength: 0.0,
                        texture_scale: 8.0,
                        tint: LinearRgba::new(0.9, 0.85, 0.5, 1.0),
                    },
                    TextureLayerConfig {
                        texture_path: "textures/grass.png".to_string(),
                        start_height: 0.35,
                        blend_strength: 0.15,
                        tint_strength: 0.0,
                        texture_scale: 12.0,
                        tint: LinearRgba::new(0.2, 0.7, 0.1, 1.0),
                    },
                    TextureLayerConfig {
                        texture_path: "textures/rock.png".to_string(),
                        start_height: 0.8,
                        blend_strength: 0.1,
                        tint_strength: 0.0,
                        texture_scale: 20.0,
                        tint: LinearRgba::new(0.5, 0.5, 0.5, 1.0),
                    },
                ],
            },
        }
    }
}

fn default_endless_lod_bands() -> Vec<EndlessTerrainLodBand> {
    vec![
        EndlessTerrainLodBand::new(0, 300.0),
        EndlessTerrainLodBand::new(1, 800.0),
        EndlessTerrainLodBand::new(2, 1800.0),
        EndlessTerrainLodBand::new(3, 3200.0),
    ]
}

impl MapGenerator {
    const MIN_VISIBLE_DISTANCE: f32 = 1.0;
    pub(crate) fn set_falloff_map(&mut self, falloff_map: Vec<Vec<f32>>) -> bool {
        if self.falloff_map == falloff_map {
            return false;
        }

        self.falloff_map = falloff_map;
        true
    }
    pub fn max_endless_visible_distance(&self) -> f32 {
        self.endless_lod_bands
            .last()
            .map(|band| band.visible_distance.max(Self::MIN_VISIBLE_DISTANCE))
            .unwrap_or(Self::MIN_VISIBLE_DISTANCE)
    }
}

pub struct MapData {
    pub noise_map: Vec<Vec<f32>>,
}

impl MapData {
    pub fn new(noise_map: Vec<Vec<f32>>) -> Self {
        Self { noise_map }
    }
}
