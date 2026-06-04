use crate::{persistence, terrain::MapControlsPlugin};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_persistent::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{RngExt, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

pub struct PerlinMapPlugin;

#[derive(Component)]
struct GroundPlane;

#[derive(Resource, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapConfigs {
    pub height: u32,
    pub width: u32,
    pub scale: f64,
    pub octaves: u32,
    pub persistence: f64,
    pub lacunarity: f64,
    pub frequency: f64,
    pub seed: u32,
    pub offset_x: f64,
    pub offset_y: f64,
}

impl Default for MapConfigs {
    fn default() -> Self {
        Self {
            height: 100,
            width: 100,
            scale: 4.0,
            octaves: 5,
            persistence: 0.4,
            lacunarity: 2.8,
            frequency: 1.0,
            seed: 0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

impl MapConfigs {
    fn sanitized(self) -> Self {
        let defaults = Self::default();

        if self.looks_like_legacy_clamped_default() {
            return defaults;
        }

        Self {
            height: valid_non_zero_or(self.height, defaults.height),
            width: valid_non_zero_or(self.width, defaults.width),
            scale: valid_positive_or(self.scale, defaults.scale),
            octaves: valid_non_zero_or(self.octaves, defaults.octaves),
            persistence: valid_non_negative_or(self.persistence, defaults.persistence),
            lacunarity: valid_positive_or(self.lacunarity, defaults.lacunarity),
            frequency: valid_positive_or(self.frequency, defaults.frequency),
            seed: self.seed,
            offset_x: valid_finite_or(self.offset_x, defaults.offset_x),
            offset_y: valid_finite_or(self.offset_y, defaults.offset_y),
        }
    }

    fn looks_like_legacy_clamped_default(self) -> bool {
        let defaults = Self::default();

        self.height == 1
            && self.width == 1
            && self.octaves == 1
            && approx_eq(self.scale, defaults.scale)
            && (approx_eq(self.persistence, 0.0) || approx_eq(self.persistence, defaults.persistence))
            && approx_eq(self.lacunarity, defaults.lacunarity)
            && approx_eq(self.frequency, defaults.frequency)
            && self.seed == defaults.seed
            && approx_eq(self.offset_x, defaults.offset_x)
            && approx_eq(self.offset_y, defaults.offset_y)
    }
}

impl Plugin for PerlinMapPlugin {
    fn build(&self, app: &mut App) {
        let persistence = persistence::PersistenceConfig::new("map_configs");
        let mut map_configs = persistence.get_resource::<MapConfigs>("map configs", "map_configs.bin");
        let sanitized_map_configs = map_configs.sanitized();

        if *map_configs != sanitized_map_configs {
            map_configs
                .set(sanitized_map_configs)
                .expect("failed to sanitize persisted map configs");
        }

        app.insert_resource(map_configs)
        .add_plugins(MapControlsPlugin)
        .add_systems(Startup, setup_noise_plane)
        .add_systems(
            Update,
            refresh_noise_plane.run_if(resource_changed::<Persistent<MapConfigs>>),
        );
    }
}

fn valid_positive_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn valid_non_zero_or(value: u32, fallback: u32) -> u32 {
    if value > 0 {
        value
    } else {
        fallback
    }
}

fn valid_non_negative_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && value >= 0.0 {
        value
    } else {
        fallback
    }
}

fn valid_finite_or(value: f64, fallback: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= f64::EPSILON
}

fn create_noise_texture(map_configs: &MapConfigs) -> Image {
    let width = map_configs.width;
    let height = map_configs.height;

    let noise_map = generate_noise_map(map_configs);

    let mut texture_data = Vec::with_capacity(noise_map.len() * 4);

    for noise_value in noise_map {
        let color_byte = (noise_value.clamp(0.0, 1.0) * 255.0) as u8;

        texture_data.push(color_byte);
        texture_data.push(color_byte);
        texture_data.push(color_byte);
        texture_data.push(255);
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

fn generate_noise_map(map_configs: &MapConfigs) -> Vec<f64> {
    let width = map_configs.width as usize;
    let height = map_configs.height as usize;
    let scale = map_configs.scale.max(0.0001);
    let octaves = map_configs.octaves.max(1) as usize;
    let half_width = map_configs.width as f64 / 2.0;
    let half_height = map_configs.height as f64 / 2.0;
    let perlin = Perlin::new(0);
    let octave_offsets = build_octave_offsets(map_configs, octaves);

    let mut noise_map = vec![0.0; width * height];
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
            noise_map[y * width + x] = noise_height;
        }
    }

    let noise_range = max_noise_height - min_noise_height;
    for value in &mut noise_map {
        *value = if noise_range.abs() <= f64::EPSILON {
            0.0
        } else {
            ((*value - min_noise_height) / noise_range).clamp(0.0, 1.0)
        };
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
