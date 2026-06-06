use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapConfigs {
    pub height: u32,
    pub width: u32,
    pub scale: f64,
    pub octaves: u32,
    pub persistence: f64,
    pub lacunarity: f64,
    pub frequency: f64,
    pub seed: u32,
    pub offset_x: f64,
    pub offset_y: f64,
    pub draw_mode: DrawMode,
    pub regions: Vec<TerrainType>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DrawMode {
    NoiseMap,
    ColorMap,
}

impl DrawMode {
    pub const ALL: [Self; 2] = [Self::NoiseMap, Self::ColorMap];

    pub fn label(self) -> &'static str {
        match self {
            Self::NoiseMap => "Noise Map",
            Self::ColorMap => "Color Map",
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
            height: 100,
            width: 100,
            scale: 4.0,
            octaves: 5,
            persistence: 0.4,
            lacunarity: 2.8,
            frequency: 1.0,
            seed: 0,
            offset_x: 0.0,
            offset_y: 0.0,
            draw_mode: DrawMode::default(),
            regions: vec![],
        }
    }
}

impl MapConfigs {
    const SCALE_STEP: f64 = 1.0;
    const FLOAT_STEP: f64 = 0.1;
    const OFFSET_STEP: f64 = 1.0;

    pub(crate) fn sanitized(&self) -> Self {
        let defaults = Self::default();

        if self.looks_like_legacy_clamped_default() {
            return defaults;
        }

        Self {
            height: valid_non_zero_or(self.height, defaults.height),
            width: valid_non_zero_or(self.width, defaults.width),
            scale: valid_positive_or(self.scale, defaults.scale),
            octaves: valid_non_zero_or(self.octaves, defaults.octaves),
            persistence: valid_non_negative_or(self.persistence, defaults.persistence),
            lacunarity: valid_positive_or(self.lacunarity, defaults.lacunarity),
            frequency: valid_positive_or(self.frequency, defaults.frequency),
            seed: self.seed,
            offset_x: valid_finite_or(self.offset_x, defaults.offset_x),
            offset_y: valid_finite_or(self.offset_y, defaults.offset_y),
            draw_mode: self.draw_mode,
            regions: if self.regions.is_empty() {
                defaults.regions
            } else {
                self.regions.clone()
            },
        }
    }

    fn looks_like_legacy_clamped_default(&self) -> bool {
        let defaults = Self::default();

        self.height == 1
            && self.width == 1
            && self.octaves == 1
            && approx_eq(self.scale, defaults.scale)
            && (approx_eq(self.persistence, 0.0)
                || approx_eq(self.persistence, defaults.persistence))
            && approx_eq(self.lacunarity, defaults.lacunarity)
            && approx_eq(self.frequency, defaults.frequency)
            && self.seed == defaults.seed
            && approx_eq(self.offset_x, defaults.offset_x)
            && approx_eq(self.offset_y, defaults.offset_y)
    }

    pub fn set_height(&mut self, height: u32) -> bool {
        set_non_zero_value(&mut self.height, height)
    }

    pub fn set_width(&mut self, width: u32) -> bool {
        set_non_zero_value(&mut self.width, width)
    }

    pub fn decrement_height(&mut self) -> bool {
        decrement_dimension(&mut self.height)
    }

    pub fn increment_height(&mut self) -> bool {
        increment_dimension(&mut self.height)
    }

    pub fn decrement_width(&mut self) -> bool {
        decrement_dimension(&mut self.width)
    }

    pub fn increment_width(&mut self) -> bool {
        increment_dimension(&mut self.width)
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

    pub fn set_draw_mode(&mut self, draw_mode: DrawMode) -> bool {
        if self.draw_mode == draw_mode {
            return false;
        }

        self.draw_mode = draw_mode;
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
