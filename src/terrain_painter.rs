pub mod brush;
pub mod custom_cursor;
pub mod sculpt_map;
pub mod sync_grid;
pub mod undo_redo;

use bevy::prelude::*;

use crate::terrain::endless_terrain::{sync_endless_terrain};

pub use brush::{BrushConfig, TerrainTool};
pub use custom_cursor::{draw_custom_cursor};
pub use sculpt_map::{SculptMap, sculpt_paint_system, ray_intersect_terrain, sample_total_height};
pub use sync_grid::{SyncGridRequest, sync_grid_system};
pub use undo_redo::{UndoStack, UndoEntry, undo_redo_system};
pub struct TerrainPainterPlugin;

impl Plugin for TerrainPainterPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SculptMap::default())
            .insert_resource(BrushConfig::default())
            .insert_resource(SyncGridRequest::default())
            .insert_resource(UndoStack::default())
            .add_systems(Update, sculpt_paint_system.before(sync_endless_terrain))
            .add_systems(Update, draw_custom_cursor)
            .add_systems(Update, sync_grid_system)
            .add_systems(
                Update,
                undo_redo_system.before(sculpt_paint_system),
            );
    }
}




