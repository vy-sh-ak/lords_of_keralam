

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{terrain::MapGenerator, terrain_painter::SculptMap};

#[derive(Serialize, Deserialize)]
pub struct MapMetadata {
    pub name: String,
}


#[derive(Resource, Default)]
pub struct ActiveMap {
    pub current_map: Option<String>,
}

#[derive(Resource, Clone, Serialize, Deserialize, PartialEq, Reflect)]
pub struct DefaultMapConfig {
    pub default_map: Option<String>,
}

impl Default for DefaultMapConfig {
    fn default() -> Self {
        Self { default_map: None }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MapAsset {
    pub metadata: MapMetadata,
    pub map_generator: MapGenerator,
    pub sculpt_map: SculptMap,
}
