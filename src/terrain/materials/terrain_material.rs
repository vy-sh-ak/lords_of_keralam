use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

use crate::terrain::{MapGenerator, data::TextureLayerConfig};

const SHADER_ASSET_PATH: &str = "shaders/terrain_material.wgsl";
const MAX_TEXTURES: usize = 4;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub params: Vec4,
    #[uniform(0)]
    pub layer0: Vec4,
    #[uniform(0)]
    pub layer1: Vec4,
    #[uniform(0)]
    pub layer2: Vec4,
    #[uniform(0)]
    pub layer3: Vec4,
    #[uniform(0)]
    pub tint0: Vec4,
    #[uniform(0)]
    pub tint1: Vec4,
    #[uniform(0)]
    pub tint2: Vec4,
    #[uniform(0)]
    pub tint3: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub texture0: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub texture1: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    pub texture2: Handle<Image>,
    #[texture(7)]
    #[sampler(8)]
    pub texture3: Handle<Image>,
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

fn pack_layer(layer: &TextureLayerConfig, height_multiplier: f32) -> Vec4 {
    Vec4::new(
        layer.start_height * height_multiplier,
        layer.blend_strength * height_multiplier,
        layer.tint_strength,
        0.0,
    )
}

fn pack_tint(layer: &TextureLayerConfig) -> Vec4 {
    Vec4::new(layer.tint.red, layer.tint.green, layer.tint.blue, layer.tint.alpha)
}

pub fn build_terrain_material(
    materials: &mut Assets<TerrainMaterial>,
    asset_server: &AssetServer,
    map_generator: &MapGenerator,
) -> Handle<TerrainMaterial> {
    let texture_data = &map_generator.texture_data;
    let height_multiplier = map_generator.terrain_data.height_multiplier;

    let mut sorted: Vec<&TextureLayerConfig> = texture_data.layers.iter().collect();
    sorted.sort_by(|a, b| a.start_height.partial_cmp(&b.start_height).unwrap());
    let count = sorted.len().min(MAX_TEXTURES);

    let texture_scale = sorted.first().map_or(0.2, |l| 1.0 / l.texture_scale);

    let mut layers = [Vec4::ZERO; MAX_TEXTURES];
    let mut tints = [Vec4::ZERO; MAX_TEXTURES];
    let mut textures: [Handle<Image>; MAX_TEXTURES] = core::array::from_fn(|_| Handle::default());

    for (i, layer) in sorted.iter().take(MAX_TEXTURES).enumerate() {
        layers[i] = pack_layer(layer, height_multiplier);
        tints[i] = pack_tint(layer);
        textures[i] = asset_server.load(&layer.texture_path);
    }

    let handle = materials.add(TerrainMaterial {
        params: Vec4::new(texture_scale, texture_data.min_height, texture_data.max_height, count as f32),
        layer0: layers[0],
        layer1: layers[1],
        layer2: layers[2],
        layer3: layers[3],
        tint0: tints[0],
        tint1: tints[1],
        tint2: tints[2],
        tint3: tints[3],
        texture0: textures[0].clone(),
        texture1: textures[1].clone(),
        texture2: textures[2].clone(),
        texture3: textures[3].clone(),
    });

    info!("Built TerrainMaterial handle={:?} with {} layers", handle, count);
    handle
}
