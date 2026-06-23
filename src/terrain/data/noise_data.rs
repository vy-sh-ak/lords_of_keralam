use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Reflect, InspectorOptions, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[reflect(InspectorOptions)]
pub struct NoiseData {

    #[inspector(min = 0.1, max = 10.0)]
    pub frequency: f64,

    #[inspector(min = 0.0, max = 1000.0)]
    pub scale: f64,

    #[inspector(min = 1, max = 10)]
    pub octaves: u32,

    #[inspector(min = 0.0, max = 5.0)]
    pub lacunarity: f64,

    #[inspector(min = 0.0, max = 5.0)]
    pub persistence: f64,

    
    pub offset_x: f64,
    pub offset_y: f64,

    pub seed: u32,
}
