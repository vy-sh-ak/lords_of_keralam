use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    math::VectorSpace,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::terrain::TextureData;

// pub fn texture_from_color_map(color_map: &[Vec<[u8; 4]>]) -> Image {
//     let height = color_map.len() as u32;
//     let width = color_map.first().map_or(0, |row| row.len()) as u32;
//     let texture_data = Vec::with_capacity((width as usize) * (height as usize) * 4);
//     draw_texture(texture_data, width, height)
// }

pub fn white_texture(width: u32, height: u32) -> Image {
    let texture_data = vec![255u8; (width as usize) * (height as usize) * 4];
    draw_texture(texture_data, width, height)
}

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

pub fn texture_from_terrain_colors(
    height_map: &[Vec<f32>],
    texture_data: &TextureData,
) -> Image {
    let height = height_map.len() as u32;
    let width = height_map.first().map_or(0, |row| row.len()) as u32;
    let mut bytes = Vec::with_capacity((width as usize) * (height as usize) * 4);

    let colors = &texture_data.base_colors;
    let thresholds = &texture_data.base_start_heights;

    for row in height_map {
        for &noise_value in row {
            let h = noise_value.clamp(0.0, 1.0);
            let color = sample_color(colors, thresholds, h);
            bytes.extend_from_slice(&[
                (color.red * 255.0) as u8,
                (color.green * 255.0) as u8,
                (color.blue * 255.0) as u8,
                (color.alpha * 255.0) as u8,
            ]);
        }
    }

    draw_texture(bytes, width, height)
}

fn sample_color(colors: &[LinearRgba], thresholds: &[f32], t: f32) -> LinearRgba {
    if colors.is_empty() {
        return LinearRgba::WHITE;
    }
    if t <= thresholds[0] {
        return colors[0];
    }
    for i in 0..thresholds.len().saturating_sub(1) {
        if t >= thresholds[i] && t < thresholds[i + 1] {
            let blend =
                ((t - thresholds[i]) / (thresholds[i + 1] - thresholds[i])).clamp(0.0, 1.0);
            return colors[i].lerp(colors[i + 1], blend);
        }
    }
    *colors.last().unwrap()
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
    image.sampler = ImageSampler::nearest();
    image
}
