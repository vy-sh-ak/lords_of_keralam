use bevy::prelude::*;
use bevy_persistent::Persistent;
use crate::terrain::EndlessTerrainState;
use crate::terrain::MapGenerator;
use crate::terrain_painter::{SculptMap, SyncGridRequest, UndoStack};

use super::{ActiveMap, DefaultMapConfig, MapAsset, MapAssetService};
/// Apply a loaded MapAsset to the world, updating all relevant resources.
pub fn apply_map_asset_to_world(world: &mut World, asset: MapAsset, name: &str) {
    let mut persistent = world.resource_mut::<Persistent<MapGenerator>>();
    *persistent.get_mut() = asset.map_generator;
    persistent.set_changed();

    let mut sculpt = world.resource_mut::<SculptMap>();
    sculpt.chunks = asset.sculpt_map.chunks;
    let chunk_keys: Vec<_> = sculpt.chunks.keys().copied().collect();
    drop(sculpt);

    let mut endless = world.resource_mut::<EndlessTerrainState>();
    for coord in chunk_keys {
        endless.mark_chunk_dirty(coord);
    }

    world.resource_mut::<UndoStack>().undo_entries.clear();
    world.resource_mut::<UndoStack>().redo_entries.clear();
    world.resource_mut::<SyncGridRequest>().0 = true;
    world.resource_mut::<ActiveMap>().current_map = Some(name.to_string());
}

/// Try to auto-load the default map at startup. If it fails, clear the default.
pub fn auto_load_default_map(world: &mut World) {
    let default_map = {
        let config = world.resource::<Persistent<DefaultMapConfig>>();
        config.default_map.clone()
    };

    let Some(ref name) = default_map else {
        return;
    };

    match MapAssetService::load(name) {
        Ok(asset) => {
            info!("Auto-loading default map: {name}");
            apply_map_asset_to_world(world, asset, name);
        }
        Err(e) => {
            warn!("Failed to load default map '{name}': {e}. Clearing default.");
            let mut config = world.resource_mut::<Persistent<DefaultMapConfig>>();
            config.default_map = None;
            config.set_changed();
        }
    }
}
