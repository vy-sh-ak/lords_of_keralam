use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Reflect, InspectorOptions, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(InspectorOptions)]
pub struct TextureData {
    pub base_colors: Vec<LinearRgba>,
    pub base_start_heights: Vec<f32>,

    pub min_height: f32,
    pub max_height: f32,
}