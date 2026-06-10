use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use super::HeightCurve;

#[derive(Resource, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapConfigs {
    pub map_chunk_size: u32,
    pub level_of_detail: u32,
    pub scale: f64,
    pub octaves: u32,
    pub persistence: f64,
    pub lacunarity: f64,
    pub frequency: f64,
    pub seed: u32,
    pub offset_x: f64,
    pub offset_y: f64,
    pub draw_mode: DrawMode,
    #[serde(default)]
    pub show_uv_wireframe: bool,
    pub height_multiplier: f32,
    #[serde(default)]
    pub height_curve: HeightCurve,
    #[serde(default = "default_endless_lod_bands")]
    pub endless_lod_bands: Vec<EndlessTerrainLodBand>,
    pub regions: Vec<TerrainType>,

    pub use_falloff_map: bool,
    pub falloff_map: Vec<Vec<f32>>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DrawMode {
    NoiseMap,
    ColorMap,
    Mesh,
    EndlessTerrain,
    FallOffMap
}

impl DrawMode {
    pub const ALL: [Self; 5] = [Self::NoiseMap, Self::ColorMap, Self::Mesh, Self::EndlessTerrain, Self::FallOffMap];

    pub fn label(self) -> &'static str {
        match self {
            Self::NoiseMap => "Noise Map",
            Self::ColorMap => "Color Map",
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TerrainType {
    pub name: String,
    pub height: f64,
    pub color: (u8, u8, u8),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EndlessTerrainLodBand {
    pub level_of_detail: u32,
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

impl Default for TerrainType {
    fn default() -> Self {
        Self {
            name: String::new(),
            height: 0.0,
            color: (0, 0, 0),
        }
    }
}

impl Default for MapConfigs {
    fn default() -> Self {
        Self {
            map_chunk_size: 239,
            level_of_detail: 0,
            scale: 4.0,
            octaves: 5,
            persistence: 0.4,
            lacunarity: 2.8,
            frequency: 1.0,
            seed: 0,
            offset_x: 0.0,
            offset_y: 0.0,
            draw_mode: DrawMode::default(),
            show_uv_wireframe: false,
            height_multiplier: 1.0,
            height_curve: HeightCurve::default(),
            endless_lod_bands: default_endless_lod_bands(),
            regions: vec![],
            use_falloff_map: false,
            falloff_map: vec![],
        }
    }
}

impl MapConfigs {
    const MIN_LEVEL_OF_DETAIL: u32 = 0;
    const MAX_LEVEL_OF_DETAIL: u32 = 6;
    const MIN_VISIBLE_DISTANCE: f32 = 1.0;
    const SCALE_STEP: f64 = 1.0;
    const FLOAT_STEP: f64 = 0.1;
    const HEIGHT_MULTIPLIER_STEP: f32 = 0.1;
    const OFFSET_STEP: f64 = 1.0;

    pub(crate) fn sanitized(&self) -> Self {
        let defaults = Self::default();

        if self.looks_like_legacy_clamped_default() {
            return defaults;
        }

        Self {
            map_chunk_size: valid_non_zero_or(self.map_chunk_size, defaults.map_chunk_size),
            level_of_detail: valid_inclusive_u32_or(
                self.level_of_detail,
                Self::MIN_LEVEL_OF_DETAIL,
                Self::MAX_LEVEL_OF_DETAIL,
                defaults.level_of_detail,
            ),
            scale: valid_positive_or(self.scale, defaults.scale),
            octaves: valid_non_zero_or(self.octaves, defaults.octaves),
            persistence: valid_non_negative_or(self.persistence, defaults.persistence),
            lacunarity: valid_positive_or(self.lacunarity, defaults.lacunarity),
            frequency: valid_positive_or(self.frequency, defaults.frequency),
            seed: self.seed,
            offset_x: valid_finite_or(self.offset_x, defaults.offset_x),
            offset_y: valid_finite_or(self.offset_y, defaults.offset_y),
            draw_mode: self.draw_mode,
            show_uv_wireframe: self.show_uv_wireframe,
            height_multiplier: if self.height_multiplier.is_finite() && self.height_multiplier > 0.0 {
                self.height_multiplier
            } else {
                defaults.height_multiplier
            },
            height_curve: self.height_curve.sanitized(),
            endless_lod_bands: sanitize_endless_lod_bands(&self.endless_lod_bands),
            regions: if self.regions.is_empty() {
                defaults.regions
            } else {
                self.regions.clone()
            },
            use_falloff_map: self.use_falloff_map,
            falloff_map: if self.falloff_map.is_empty() {
                defaults.falloff_map
            } else {
                self.falloff_map.clone()
            },
        }
    }

    fn looks_like_legacy_clamped_default(&self) -> bool {
        let defaults = Self::default();

        self.map_chunk_size == 1
            && self.octaves == 1
            && approx_eq(self.scale, defaults.scale)
            && (approx_eq(self.persistence, 0.0)
                || approx_eq(self.persistence, defaults.persistence))
            && approx_eq(self.lacunarity, defaults.lacunarity)
            && approx_eq(self.frequency, defaults.frequency)
            && self.seed == defaults.seed
            && approx_eq(self.offset_x, defaults.offset_x)
            && approx_eq(self.offset_y, defaults.offset_y)
            && approx_eq(self.height_multiplier as f64, defaults.height_multiplier as f64)
    }

    pub fn set_map_chunk_size(&mut self, map_chunk_size: u32) -> bool {
        set_non_zero_value(&mut self.map_chunk_size, map_chunk_size)
    }

    pub fn decrement_map_chunk_size(&mut self) -> bool {
        decrement_dimension(&mut self.map_chunk_size)
    }

    pub fn increment_map_chunk_size(&mut self) -> bool {
        increment_dimension(&mut self.map_chunk_size)
    }

    pub fn set_level_of_detail(&mut self, level_of_detail: u32) -> bool {
        set_u32_inclusive_value(
            &mut self.level_of_detail,
            level_of_detail,
            Self::MIN_LEVEL_OF_DETAIL,
            Self::MAX_LEVEL_OF_DETAIL,
        )
    }

    pub fn decrement_level_of_detail(&mut self) -> bool {
        if self.level_of_detail > Self::MIN_LEVEL_OF_DETAIL {
            let next_level_of_detail = self.level_of_detail - 1;
            set_u32_value(&mut self.level_of_detail, next_level_of_detail)
        } else {
            false
        }
    }

    pub fn increment_level_of_detail(&mut self) -> bool {
        if self.level_of_detail < Self::MAX_LEVEL_OF_DETAIL {
            let next_level_of_detail = self.level_of_detail + 1;
            set_u32_value(&mut self.level_of_detail, next_level_of_detail)
        } else {
            false
        }
    }

    pub fn set_scale(&mut self, scale: f64) -> bool {
        set_positive_value(&mut self.scale, scale)
    }

    pub fn decrement_scale(&mut self) -> bool {
        if self.scale > Self::SCALE_STEP {
            let next_scale = self.scale - Self::SCALE_STEP;
            set_f64_value(&mut self.scale, next_scale)
        } else {
            false
        }
    }

    pub fn increment_scale(&mut self) -> bool {
        let next_scale = self.scale + Self::SCALE_STEP;
        set_f64_value(&mut self.scale, next_scale)
    }

    pub fn set_seed(&mut self, seed: u32) -> bool {
        set_u32_value(&mut self.seed, seed)
    }

    pub fn decrement_seed(&mut self) -> bool {
        let next_seed = self.seed.saturating_sub(1);
        set_u32_value(&mut self.seed, next_seed)
    }

    pub fn increment_seed(&mut self) -> bool {
        let next_seed = self.seed.saturating_add(1);
        set_u32_value(&mut self.seed, next_seed)
    }

    pub fn set_offset_x(&mut self, offset_x: f64) -> bool {
        set_finite_value(&mut self.offset_x, offset_x)
    }

    pub fn decrement_offset_x(&mut self) -> bool {
        let next_offset_x = self.offset_x - Self::OFFSET_STEP;
        set_f64_value(&mut self.offset_x, next_offset_x)
    }

    pub fn increment_offset_x(&mut self) -> bool {
        let next_offset_x = self.offset_x + Self::OFFSET_STEP;
        set_f64_value(&mut self.offset_x, next_offset_x)
    }

    pub fn set_offset_y(&mut self, offset_y: f64) -> bool {
        set_finite_value(&mut self.offset_y, offset_y)
    }

    pub fn decrement_offset_y(&mut self) -> bool {
        let next_offset_y = self.offset_y - Self::OFFSET_STEP;
        set_f64_value(&mut self.offset_y, next_offset_y)
    }

    pub fn increment_offset_y(&mut self) -> bool {
        let next_offset_y = self.offset_y + Self::OFFSET_STEP;
        set_f64_value(&mut self.offset_y, next_offset_y)
    }

    pub fn set_persistence(&mut self, persistence: f64) -> bool {
        set_non_negative_value(&mut self.persistence, persistence)
    }

    pub fn decrement_persistence(&mut self) -> bool {
        let next_persistence = (self.persistence - Self::FLOAT_STEP).max(0.0);
        set_f64_value(&mut self.persistence, next_persistence)
    }

    pub fn increment_persistence(&mut self) -> bool {
        let next_persistence = self.persistence + Self::FLOAT_STEP;
        set_f64_value(&mut self.persistence, next_persistence)
    }

    pub fn set_lacunarity(&mut self, lacunarity: f64) -> bool {
        set_non_negative_value(&mut self.lacunarity, lacunarity)
    }

    pub fn decrement_lacunarity(&mut self) -> bool {
        let next_lacunarity = (self.lacunarity - Self::FLOAT_STEP).max(0.0);
        set_f64_value(&mut self.lacunarity, next_lacunarity)
    }

    pub fn increment_lacunarity(&mut self) -> bool {
        let next_lacunarity = self.lacunarity + Self::FLOAT_STEP;
        set_f64_value(&mut self.lacunarity, next_lacunarity)
    }

    pub fn set_frequency(&mut self, frequency: f64) -> bool {
        set_minimum_value(&mut self.frequency, frequency, Self::FLOAT_STEP)
    }

    pub fn decrement_frequency(&mut self) -> bool {
        let next_frequency = (self.frequency - Self::FLOAT_STEP).max(Self::FLOAT_STEP);
        set_f64_value(&mut self.frequency, next_frequency)
    }

    pub fn increment_frequency(&mut self) -> bool {
        let next_frequency = self.frequency + Self::FLOAT_STEP;
        set_f64_value(&mut self.frequency, next_frequency)
    }

    pub fn set_height_multiplier(&mut self, height_multiplier: f32) -> bool {
        set_positive_f32_value(&mut self.height_multiplier, height_multiplier)
    }

    pub fn decrement_height_multiplier(&mut self) -> bool {
        let next_height_multiplier =
            (self.height_multiplier - Self::HEIGHT_MULTIPLIER_STEP).max(Self::HEIGHT_MULTIPLIER_STEP);
        set_f32_value(&mut self.height_multiplier, next_height_multiplier)
    }

    pub fn increment_height_multiplier(&mut self) -> bool {
        let next_height_multiplier = self.height_multiplier + Self::HEIGHT_MULTIPLIER_STEP;
        set_f32_value(&mut self.height_multiplier, next_height_multiplier)
    }

    pub fn add_height_curve_point(&mut self) -> bool {
        self.height_curve.add_point()
    }

    pub fn remove_height_curve_point(&mut self, index: usize) -> bool {
        self.height_curve.remove_point(index)
    }

    pub fn reset_height_curve(&mut self) -> bool {
        self.height_curve.reset()
    }

    pub fn set_height_curve_point_input(&mut self, index: usize, input: f32) -> bool {
        self.height_curve.set_point_input(index, input)
    }

    pub fn set_height_curve_point_output(&mut self, index: usize, output: f32) -> bool {
        self.height_curve.set_point_output(index, output)
    }

    pub fn add_endless_lod_band(&mut self) -> bool {
        let next_band = self
            .endless_lod_bands
            .last()
            .cloned()
            .map(|band| {
                EndlessTerrainLodBand::new(
                    band.level_of_detail,
                    band.visible_distance + self.map_chunk_size.max(2) as f32,
                )
            })
            .unwrap_or_else(|| default_endless_lod_bands()[0].clone());

        self.endless_lod_bands.push(next_band);
        true
    }

    pub fn remove_endless_lod_band(&mut self, index: usize) -> bool {
        if self.endless_lod_bands.len() <= 1 || index >= self.endless_lod_bands.len() {
            return false;
        }

        self.endless_lod_bands.remove(index);
        true
    }

    pub fn reset_endless_lod_bands(&mut self) -> bool {
        let defaults = default_endless_lod_bands();
        if self.endless_lod_bands == defaults {
            return false;
        }

        self.endless_lod_bands = defaults;
        true
    }

    pub fn set_endless_lod_band_distance(&mut self, index: usize, visible_distance: f32) -> bool {
        if !visible_distance.is_finite() || visible_distance < Self::MIN_VISIBLE_DISTANCE {
            return false;
        }

        let previous = index
            .checked_sub(1)
            .and_then(|previous_index| self.endless_lod_bands.get(previous_index))
            .map(|previous| previous.visible_distance)
            .unwrap_or(Self::MIN_VISIBLE_DISTANCE);
        let next = self
            .endless_lod_bands
            .get(index + 1)
            .map(|next_band| next_band.visible_distance)
            .unwrap_or(f32::MAX);
        let clamped = visible_distance.clamp(previous, next);

        let Some(band) = self.endless_lod_bands.get_mut(index) else {
            return false;
        };

        if approx_eq_f32(band.visible_distance, clamped) {
            return false;
        }

        band.visible_distance = clamped;
        true
    }

    pub fn set_endless_lod_band_level_of_detail(
        &mut self,
        index: usize,
        level_of_detail: u32,
    ) -> bool {
        let Some(band) = self.endless_lod_bands.get_mut(index) else {
            return false;
        };

        if level_of_detail < Self::MIN_LEVEL_OF_DETAIL || level_of_detail > Self::MAX_LEVEL_OF_DETAIL {
            return false;
        }

        if band.level_of_detail == level_of_detail {
            return false;
        }

        band.level_of_detail = level_of_detail;
        true
    }

    pub fn max_endless_visible_distance(&self) -> f32 {
        self.endless_lod_bands
            .last()
            .map(|band| band.visible_distance.max(Self::MIN_VISIBLE_DISTANCE))
            .unwrap_or(Self::MIN_VISIBLE_DISTANCE)
    }

    pub fn set_draw_mode(&mut self, draw_mode: DrawMode) -> bool {
        if self.draw_mode == draw_mode {
            return false;
        }

        self.draw_mode = draw_mode;
        true
    }

    pub fn set_show_uv_wireframe(&mut self, show_uv_wireframe: bool) -> bool {
        if self.show_uv_wireframe == show_uv_wireframe {
            return false;
        }

        self.show_uv_wireframe = show_uv_wireframe;
        true
    }

    pub fn set_region_count(&mut self, count: usize) -> bool {
        if self.regions.len() == count {
            return false;
        }

        self.regions.resize(count, TerrainType::default());
        true
    }

    pub fn decrement_region_count(&mut self) -> bool {
        if self.regions.is_empty() {
            return false;
        }

        self.regions.pop();
        true
    }

    pub fn increment_region_count(&mut self) -> bool {
        self.regions.push(TerrainType::default());
        true
    }

    pub fn set_region_name(&mut self, index: usize, name: String) -> bool {
        let Some(region) = self.regions.get_mut(index) else {
            return false;
        };

        if region.name == name {
            return false;
        }

        region.name = name;
        true
    }

    pub fn set_region_height(&mut self, index: usize, height: f64) -> bool {
        let Some(region) = self.regions.get_mut(index) else {
            return false;
        };

        if !height.is_finite() || approx_eq(region.height, height) {
            return false;
        }

        region.height = height;
        true
    }

    pub fn set_region_color(&mut self, index: usize, color: (u8, u8, u8)) -> bool {
        let Some(region) = self.regions.get_mut(index) else {
            return false;
        };

        if region.color == color {
            return false;
        }

        region.color = color;
        true
    }

    pub fn set_use_falloff_map(&mut self, use_falloff_map: bool) -> bool {
        if self.use_falloff_map == use_falloff_map {
            return false;
        }

        self.use_falloff_map = use_falloff_map;
        true
    }

    pub fn set_falloff_map(&mut self, falloff_map: Vec<Vec<f32>>) -> bool {
        if self.falloff_map == falloff_map {
            return false;
        }

        self.falloff_map = falloff_map;
        true
    }
}

fn valid_positive_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn set_non_zero_value(slot: &mut u32, value: u32) -> bool {
    if value == 0 || *slot == value {
        return false;
    }

    *slot = value;
    true
}

fn set_u32_value(slot: &mut u32, value: u32) -> bool {
    if *slot == value {
        return false;
    }

    *slot = value;
    true
}

fn set_u32_inclusive_value(slot: &mut u32, value: u32, minimum: u32, maximum: u32) -> bool {
    if value < minimum || value > maximum {
        return false;
    }

    set_u32_value(slot, value)
}

fn decrement_dimension(slot: &mut u32) -> bool {
    if *slot > 1 {
        *slot -= 1;
        true
    } else {
        false
    }
}

fn increment_dimension(slot: &mut u32) -> bool {
    *slot = slot.saturating_add(1);
    true
}

fn set_f64_value(slot: &mut f64, value: f64) -> bool {
    if approx_eq(*slot, value) {
        return false;
    }

    *slot = value;
    true
}

fn set_positive_value(slot: &mut f64, value: f64) -> bool {
    if value.is_finite() && value > 0.0 {
        set_f64_value(slot, value)
    } else {
        false
    }
}

fn set_f32_value(slot: &mut f32, value: f32) -> bool {
    if approx_eq_f32(*slot, value) {
        return false;
    }

    *slot = value;
    true
}

fn set_positive_f32_value(slot: &mut f32, value: f32) -> bool {
    if value.is_finite() && value > 0.0 {
        set_f32_value(slot, value)
    } else {
        false
    }
}

fn set_non_negative_value(slot: &mut f64, value: f64) -> bool {
    if value.is_finite() && value >= 0.0 {
        set_f64_value(slot, value)
    } else {
        false
    }
}

fn set_finite_value(slot: &mut f64, value: f64) -> bool {
    if value.is_finite() {
        set_f64_value(slot, value)
    } else {
        false
    }
}

fn set_minimum_value(slot: &mut f64, value: f64, minimum: f64) -> bool {
    if value.is_finite() && value >= minimum {
        set_f64_value(slot, value)
    } else {
        false
    }
}

fn valid_non_zero_or(value: u32, fallback: u32) -> u32 {
    if value > 0 { value } else { fallback }
}

fn valid_inclusive_u32_or(value: u32, minimum: u32, maximum: u32, fallback: u32) -> u32 {
    if value >= minimum && value <= maximum {
        value
    } else {
        fallback
    }
}

fn valid_non_negative_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && value >= 0.0 {
        value
    } else {
        fallback
    }
}

fn valid_finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() { value } else { fallback }
}

fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= f64::EPSILON
}

fn approx_eq_f32(left: f32, right: f32) -> bool {
    (left - right).abs() <= f32::EPSILON
}

fn default_endless_lod_bands() -> Vec<EndlessTerrainLodBand> {
    vec![
        EndlessTerrainLodBand::new(0, 220.0),
        EndlessTerrainLodBand::new(2, 420.0),
        EndlessTerrainLodBand::new(4, 700.0),
        EndlessTerrainLodBand::new(6, 1050.0),
    ]
}

fn sanitize_endless_lod_bands(bands: &[EndlessTerrainLodBand]) -> Vec<EndlessTerrainLodBand> {
    let defaults = default_endless_lod_bands();
    let mut sanitized: Vec<_> = bands
        .iter()
        .filter(|band| band.visible_distance.is_finite())
        .map(|band| EndlessTerrainLodBand {
            level_of_detail: band
                .level_of_detail
                .clamp(MapConfigs::MIN_LEVEL_OF_DETAIL, MapConfigs::MAX_LEVEL_OF_DETAIL),
            visible_distance: band.visible_distance.max(MapConfigs::MIN_VISIBLE_DISTANCE),
        })
        .collect();

    if sanitized.is_empty() {
        return defaults;
    }

    sanitized.sort_by(|left, right| left.visible_distance.total_cmp(&right.visible_distance));

    for index in 1..sanitized.len() {
        if sanitized[index].visible_distance <= sanitized[index - 1].visible_distance {
            sanitized[index].visible_distance = sanitized[index - 1].visible_distance + 1.0;
        }
    }

    sanitized
}
