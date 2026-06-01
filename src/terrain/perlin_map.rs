use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use noise::{NoiseFn, Perlin};
use crate::terrain::MapControlsPlugin;

pub struct PerlinMapPlugin;

#[derive(Component)]
struct GroundPlane;

#[derive(Resource, Clone, Copy)]
pub struct MapConfigs {
    pub height: u32,
    pub width: u32,
    pub scale: f64,
}

impl Plugin for PerlinMapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapConfigs {
            height: 100,
            width: 100,
            scale: 4.0,
        })
        .add_plugins(MapControlsPlugin)
        .add_systems(Startup, setup_noise_plane)
        .add_systems(Update, refresh_noise_plane.run_if(resource_changed::<MapConfigs>));
    }
}

fn create_noise_texture(map_configs: &MapConfigs) -> Image {
    let width = map_configs.width;
    let height = map_configs.height;
    let scale = map_configs.scale.max(0.001);

    let mut texture_data = Vec::with_capacity((width * height * 4) as usize);
    let perlin = Perlin::default();

    for y in 0..height {
        for x in 0..width {
            let sample_x = x as f64 / scale;
            let sample_y = y as f64 / scale;

            let noise_val = perlin.get([sample_x, sample_y]);
            let normalized_val = (noise_val + 1.0) / 2.0;
            let color_byte = (normalized_val * 255.0) as u8;

            texture_data.push(color_byte);
            texture_data.push(color_byte);
            texture_data.push(color_byte);
            texture_data.push(255);
        }
    }

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}

fn setup_noise_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    map_configs: Res<MapConfigs>,
) {
    let texture_handle = images.add(create_noise_texture(&map_configs));

    commands.spawn((
        GroundPlane,
        Name::new("GroundPlane"),
        Mesh3d(meshes.add(
            Plane3d::default()
                .mesh()
                .size(map_configs.width as f32, map_configs.height as f32),
        )),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.01, 0.0),
    ));
}

fn refresh_noise_plane(
    map_configs: Res<MapConfigs>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut ground_plane_query: Query<
        (&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>),
        With<GroundPlane>,
    >,
) {
    let Ok((mut mesh_handle, mut material_handle)) = ground_plane_query.single_mut() else {
        return;
    };

    *mesh_handle = Mesh3d(meshes.add(
        Plane3d::default()
            .mesh()
            .size(map_configs.width as f32, map_configs.height as f32),
    ));

    *material_handle = MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(images.add(create_noise_texture(&map_configs))),
        perceptual_roughness: 1.0,
        ..default()
    }));
}
