pub mod map_asset_service;
pub mod map_loader;
pub mod map_res;

use bevy::prelude::*;

use crate::editor_config::{AutosaveAppExt, EditorStateAppExt};
use crate::persistence::PersistenceConfig;

pub use map_asset_service::MapAssetService;
pub use map_loader::{apply_map_asset_to_world, auto_load_default_map};
pub use map_res::{ActiveMap, DefaultMapConfig, MapAsset, MapMetadata};

pub struct MapAssetPlugin;

impl Plugin for MapAssetPlugin {
    fn build(&self, app: &mut App) {
        let editor_persistence = PersistenceConfig::new("editor");
        let default_map_config = editor_persistence
            .get_resource::<DefaultMapConfig>("default map config", "default_map.toml");

        app.insert_resource(ActiveMap::default())
            .insert_resource(default_map_config)
            .register_type::<DefaultMapConfig>()
            .add_editor_state::<DefaultMapConfig>()
            .add_autosave::<DefaultMapConfig>()
            .add_systems(Startup, auto_load_default_map);
    }
}
