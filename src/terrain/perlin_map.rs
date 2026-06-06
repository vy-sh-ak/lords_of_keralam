use super::{
    perlin_map_texture::{texture_from_color_map, texture_from_height_map, ColorMap},
    DrawMode, MapConfigs, TerrainType,
};
use crate::{persistence, terrain::MapConfigPersistencePlugin};
use bevy::{
    prelude::*,
};
use bevy_persistent::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{RngExt, SeedableRng, rngs::StdRng};

pub struct PerlinMapPlugin;

#[derive(Component)]
struct GroundPlane;

impl Plugin for PerlinMapPlugin {
    fn build(&self, app: &mut App) {
        let persistence = persistence::PersistenceConfig::new("map_configs");
        let mut map_configs =
            persistence.get_resource::<MapConfigs>("map configs", "map_configs.bin");
        let sanitized_map_configs = map_configs.sanitized();

        if *map_configs != sanitized_map_configs {
            map_configs
                .set(sanitized_map_configs)
                .expect("failed to sanitize persisted map configs");
        }

        app.insert_resource(map_configs)
            .add_plugins(MapConfigPersistencePlugin)
            .add_systems(Startup, setup_noise_plane)
            .add_systems(
                Update,
                refresh_noise_plane.run_if(resource_changed::<Persistent<MapConfigs>>),
            );
    }
}

fn create_noise_texture(map_configs: &MapConfigs) -> Image {
    let noise_map = generate_noise_map(map_configs);

    match map_configs.draw_mode {
        DrawMode::NoiseMap => texture_from_height_map(&noise_map),
        DrawMode::ColorMap => {
            let color_map = build_color_map(&noise_map, &map_configs.regions);
            texture_from_color_map(&color_map)
        }
    }
}

fn build_color_map(noise_map: &[Vec<f32>], regions: &[TerrainType]) -> ColorMap {
    noise_map
        .iter()
        .map(|row| {
            row.iter()
                .map(|&height_value| terrain_color_for_height(height_value, regions))
                .collect()
        })
        .collect()
}

fn terrain_color_for_height(height_value: f32, regions: &[TerrainType]) -> [u8; 4] {
    for region in regions {
        if (height_value as f64) <= region.height {
            let (red, green, blue) = region.color;
            return [red, green, blue, 255];
        }
    }

    [0, 0, 0, 255]
}

fn generate_noise_map(map_configs: &MapConfigs) -> Vec<Vec<f32>> {
    let width = map_configs.width as usize;
    let height = map_configs.height as usize;
    let scale = map_configs.scale.max(0.0001);
    let octaves = map_configs.octaves.max(1) as usize;
    let half_width = map_configs.width as f64 / 2.0;
    let half_height = map_configs.height as f64 / 2.0;
    let perlin = Perlin::new(0);
    let octave_offsets = build_octave_offsets(map_configs, octaves);

    let mut noise_map = vec![vec![0.0_f32; width]; height];
    let mut max_noise_height = f64::NEG_INFINITY;
    let mut min_noise_height = f64::INFINITY;

    for y in 0..height {
        for x in 0..width {
            let mut amplitude = 1.0;
            let mut frequency = map_configs.frequency.max(0.0001);
            let mut noise_height = 0.0;

            for &(offset_x, offset_y) in &octave_offsets {
                let sample_x = ((x as f64 - half_width) / scale) * frequency + offset_x;
                let sample_y = ((y as f64 - half_height) / scale) * frequency + offset_y;

                let perlin_value = perlin.get([sample_x, sample_y]);
                noise_height += perlin_value * amplitude;

                amplitude *= map_configs.persistence;
                frequency *= map_configs.lacunarity;
            }

            max_noise_height = max_noise_height.max(noise_height);
            min_noise_height = min_noise_height.min(noise_height);
            noise_map[y][x] = noise_height as f32;
        }
    }

    let noise_range = max_noise_height - min_noise_height;
    for row in &mut noise_map {
        for value in row {
            *value = if noise_range.abs() <= f64::EPSILON {
                0.0
            } else {
                (((*value as f64 - min_noise_height) / noise_range).clamp(0.0, 1.0)) as f32
            };
        }
    }

    noise_map
}

fn build_octave_offsets(map_configs: &MapConfigs, octaves: usize) -> Vec<(f64, f64)> {
    let mut rng = StdRng::seed_from_u64(map_configs.seed as u64);

    (0..octaves)
        .map(|_| {
            (
                rng.random_range(-100_000.0..100_000.0) + map_configs.offset_x,
                rng.random_range(-100_000.0..100_000.0) + map_configs.offset_y,
            )
        })
        .collect()
}

fn setup_noise_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    map_configs: Res<Persistent<MapConfigs>>,
) {
    let texture_handle = images.add(create_noise_texture(&map_configs));

    commands.spawn((
        GroundPlane,
        Name::new("GroundPlane"),
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(map_configs.width as f32, map_configs.height as f32),
            ),
        ),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.01, 0.0),
    ));
}

fn refresh_noise_plane(
    map_configs: Res<Persistent<MapConfigs>>,
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

    *mesh_handle = Mesh3d(
        meshes.add(
            Plane3d::default()
                .mesh()
                .size(map_configs.width as f32, map_configs.height as f32),
        ),
    );

    *material_handle = MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(images.add(create_noise_texture(&map_configs))),
        perceptual_roughness: 1.0,
        ..default()
    }));
}
