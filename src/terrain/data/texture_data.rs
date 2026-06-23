use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Reflect, InspectorOptions, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[reflect(InspectorOptions)]
pub struct TextureData {
    pub min_height: f32,
    pub max_height: f32,
    pub layers: Vec<TextureLayerConfig>,
}

#[derive(Reflect, InspectorOptions, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(InspectorOptions)]
pub struct TextureLayerConfig {
    pub texture_path: String,
    pub start_height: f32,
    pub blend_strength: f32,
    pub tint_strength: f32,
    pub texture_scale: f32,
    pub tint: LinearRgba,
}

impl Default for TextureLayerConfig {
    fn default() -> Self {
        Self {
            texture_path: String::new(),
            start_height: 0.0,
            blend_strength: 0.1,
            tint_strength: 0.0,
            texture_scale: 10.0,
            tint: LinearRgba::WHITE,
        }
    }
}