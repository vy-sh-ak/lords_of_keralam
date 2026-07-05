use std::collections::HashMap;

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::building_placement::BuildingType;

#[derive(Resource)]
pub struct BuildingAssets {
    scenes: HashMap<BuildingType, Handle<Scene>>,
}

impl BuildingAssets {
    pub fn scene(&self, building_type: BuildingType) -> Option<&Handle<Scene>> {
        self.scenes.get(&building_type)
    }
}

fn load_building_assets(asset_server: &AssetServer) -> BuildingAssets {
    let mut scenes = HashMap::new();
    scenes.insert(
        BuildingType::Hut,
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("buildings/hut_v3.glb")),
    );
    BuildingAssets { scenes }
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        let asset_server = app.world().resource::<AssetServer>();
        app.insert_resource(load_building_assets(asset_server));
    }
}
