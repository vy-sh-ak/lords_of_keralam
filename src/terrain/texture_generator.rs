use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub fn texture_from_height_map(height_map: &[Vec<f32>]) -> Image {
    let height = height_map.len() as u32;
    let width = height_map.first().map_or(0, |row| row.len()) as u32;
    let mut texture_data = Vec::with_capacity((width as usize) * (height as usize) * 4);

    for row in height_map {
        for &height_value in row {
            let color_byte = (height_value.clamp(0.0, 1.0) * 255.0) as u8;
            texture_data.extend_from_slice(&[color_byte, color_byte, color_byte, 255]);
        }
    }

    draw_texture(texture_data, width, height)
}



fn draw_texture(texture_data: Vec<u8>, width: u32, height: u32) -> Image {
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}
