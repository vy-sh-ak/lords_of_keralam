use bevy::prelude::*;

use bevy_persistent::Persistent;

use crate::terrain::{MapGenerator, TerrainSampler};
use crate::world_grid_config::WorldGrid;

use super::{SculptMap};
use super::sculpt_map::{sample_total_height};


#[derive(Resource, Default)]
pub struct SyncGridRequest(pub bool);

pub fn sync_grid_system(
    mut request: ResMut<SyncGridRequest>,
    mut world_grid: ResMut<WorldGrid>,
    terrain_sampler: Res<TerrainSampler>,
    map_generator: Res<Persistent<MapGenerator>>,
    sculpt_map: Res<SculptMap>,
) {
    if !request.0 {
        return;
    }
    request.0 = false;
    sync_grid_heights(&mut world_grid, &terrain_sampler, &map_generator, &sculpt_map);
}

pub(crate) fn sync_grid_heights(
    world_grid: &mut WorldGrid,
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
    sculpt_map: &SculptMap,
) {
    for (_pos, tile) in world_grid.tiles.iter_mut() {
        let world = _pos.grid_to_world();
        tile.terrain_height =
            sample_total_height(terrain_sampler, map_generator, sculpt_map, world);
    }
}
