use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildingType {
    #[default]
    Hut,
}

#[derive(Debug, Clone, Copy)]
pub struct BuildingSize {
    pub tiles_x: u32,
    pub tiles_z: u32,
}

impl Default for BuildingSize {
    fn default() -> Self {
        Self {
            tiles_x: 2,
            tiles_z: 2,
        }
    }
}

#[derive(Resource, Default)]
pub struct BuildMode {
    pub enabled: bool,
    pub placing_building: Option<BuildingType>,
    pub building_size: BuildingSize,
}
