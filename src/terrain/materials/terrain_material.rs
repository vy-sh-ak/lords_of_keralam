use bevy::{prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef};

const SHADER_ASSET_PATH: &str = "shaders/terrain_material.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub min_height: f32,

    #[uniform(0)]
    pub max_height: f32,

    #[uniform(0)]
    pub blend_strength: f32,

    #[texture(1)]
    #[sampler(2)]
    pub sand_texture: Handle<Image>,

    #[texture(3)]
    #[sampler(4)]
    pub grass_texture: Handle<Image>,

    #[texture(5)]
    #[sampler(6)]
    pub rock_texture: Handle<Image>,

    #[texture(7)]
    #[sampler(8)]
    pub snow_texture: Handle<Image>,
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}