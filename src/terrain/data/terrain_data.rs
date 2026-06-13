use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::terrain::HeightCurve;

#[derive(Reflect, InspectorOptions, Clone, PartialEq, Serialize, Deserialize)]
#[reflect(InspectorOptions)]
pub struct TerrainData {

    pub use_falloff_map: bool,

    #[inspector(min = 0.0, max = 1000.0)]
    pub height_multiplier: f32,

    #[serde(default)]
    #[reflect(ignore)]
    pub height_curve: HeightCurve,
}
