pub mod build_mode;
pub mod ghost_placement;
pub mod highlight_system;

pub use build_mode::BuildMode;
pub use build_mode::BuildingType;

use bevy::prelude::*;

pub struct BuildingPlacementPlugin;

impl Plugin for BuildingPlacementPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BuildMode::default())
            .add_systems(Update, highlight_system::highlight_hovered_tile)
            .add_systems(Update, ghost_placement::ghost_placement_system);
    }
}
