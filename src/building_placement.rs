pub mod build_mode;
pub mod highlight_system;

pub use build_mode::BuildMode;

use bevy::prelude::*;

pub struct BuildingPlacementPlugin;

impl Plugin for BuildingPlacementPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BuildMode::default())
            .add_systems(Update, highlight_system::highlight_hovered_tile);
    }
}
