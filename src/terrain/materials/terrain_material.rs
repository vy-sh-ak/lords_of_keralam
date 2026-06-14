use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use crate::terrain::MapGenerator;

const SHADER_ASSET_PATH: &str = "shaders/terrain_material.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub params: Vec4,
    #[uniform(0)]
    pub tint_color: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub water_texture: Handle<Image>,
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

pub fn build_terrain_material(
    materials: &mut Assets<TerrainMaterial>,
    asset_server: &AssetServer,
    map_generator: &MapGenerator,
) -> Handle<TerrainMaterial> {
    let water_layer = map_generator.texture_data.layers.first();
    let height_multiplier = map_generator.terrain_data.height_multiplier;

    let texture_scale = water_layer.map_or(0.2, |l| 1.0 / l.texture_scale);

    let start_height = water_layer.map_or(0.0, |l| l.start_height);
    let blend_strength = water_layer.map_or(0.1, |l| l.blend_strength);
    let height_start = start_height * height_multiplier;
    let height_end = height_start + blend_strength * height_multiplier;

    let tint_strength = water_layer.map_or(0.0, |l| l.tint_strength);
    let tint = water_layer.map_or(LinearRgba::WHITE, |l| l.tint);

    let handle = materials.add(TerrainMaterial {
        params: Vec4::new(texture_scale, height_start, height_end, tint_strength),
        tint_color: Vec4::new(tint.red, tint.green, tint.blue, tint.alpha),
        water_texture: asset_server.load("textures/water.png"),
    });

    info!("Built TerrainMaterial handle={:?}", handle);
    handle
}
