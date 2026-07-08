pub mod config;
pub mod generation;
pub mod rendering;
pub mod height_curve;
pub mod endless_terrain;
pub mod mesh_generator;
pub mod fall_off_generator;
pub mod data;
pub mod terrain_sampler;
pub mod texture_generator;
pub mod terrain_material;
pub mod wireframe;

pub use config::*;
pub(crate) use generation::*;
pub use height_curve::*;
pub(crate) use endless_terrain::*;
pub use mesh_generator::*;
pub use fall_off_generator::*;
pub use data::*;
pub use terrain_sampler::*;
pub use terrain_material::*;

use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::camera_config::CameraSystems;
use crate::editor_config::{AutosaveAppExt, EditorStateAppExt};

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        let generator_persistence = crate::persistence::PersistenceConfig::new("map_generator");
        let map_generator = generator_persistence
            .get_resource::<MapGenerator>("map generator", "map_generator.toml");

        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default())
            .register_type::<NoiseData>()
            .register_type::<TerrainData>()
            .register_type::<TextureData>()
            .register_type::<MapGenerator>()
            .insert_resource(map_generator)
            .add_editor_state::<MapGenerator>()
            .add_autosave::<MapGenerator>()
            .insert_resource(TerrainSampler::default())
            .add_systems(Startup, rendering::setup_noise_plane)
            .add_systems(
                Update,
                rendering::refresh_noise_plane
                    .run_if(resource_changed::<Persistent<MapGenerator>>),
            )
            .insert_resource(EndlessTerrainState::default())
            .add_systems(
                Update,
                (
                    sync_endless_terrain.after(CameraSystems::UpdateState),
                    update_focus_height,
                ),
            );
    }
}