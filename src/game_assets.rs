use std::collections::HashMap;

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::building_placement::BuildingType;

#[derive(Resource)]
pub struct BuildingAssets {
    scenes: HashMap<BuildingType, Handle<Scene>>,
    meshes: HashMap<BuildingType, Handle<Mesh>>,
}

impl BuildingAssets {
    pub fn scene(&self, building_type: BuildingType) -> Option<&Handle<Scene>> {
        self.scenes.get(&building_type)
    }

    pub fn mesh(&self, building_type: BuildingType) -> Option<&Handle<Mesh>> {
        self.meshes.get(&building_type)
    }
}

fn load_building_assets(asset_server: &AssetServer) -> BuildingAssets {
    let path = "buildings/hut_v3.glb";
    let mut scenes = HashMap::new();
    scenes.insert(
        BuildingType::Hut,
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(path)),
    );
    let mut meshes = HashMap::new();
    meshes.insert(
        BuildingType::Hut,
        asset_server.load(
            GltfAssetLabel::Primitive { mesh: 0, primitive: 0 }.from_asset(path),
        ),
    );
    BuildingAssets { scenes, meshes }
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        let asset_server = app.world().resource::<AssetServer>();
        app.insert_resource(load_building_assets(asset_server));
    }
}
