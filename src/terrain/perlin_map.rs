use super::{
    DrawMode, MapConfigs, MeshGenerator, TerrainType,
    perlin_map_texture::{texture_from_color_map, texture_from_height_map},
};
use crate::{
    persistence,
    terrain::{FallOffGenerator, MapConfigPersistencePlugin, fall_off_generator},
};
use bevy::{
    pbr::wireframe::{Wireframe, WireframeColor},
    prelude::*,
};
use bevy_persistent::prelude::*;
use noise::{NoiseFn, Perlin};
use rand::{RngExt, SeedableRng, rngs::StdRng};

pub struct PerlinMapPlugin;

const MESH_DEBUG_WIREFRAME_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);

struct RenderAssets {
    mesh: Mesh,
    texture: Image,
    vertical_offset: f32,
    show_wireframe: bool,
}

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

fn create_render_assets(map_configs: &MapConfigs) -> RenderAssets {
    let map_data = generate_map_data(map_configs, IVec2::ZERO);

    match map_configs.draw_mode {
        DrawMode::NoiseMap => RenderAssets {
            mesh: create_plane_mesh(map_configs),
            texture: texture_from_height_map(&map_data.noise_map),
            vertical_offset: -0.01,
            show_wireframe: false,
        },
        DrawMode::ColorMap => {
            RenderAssets {
                mesh: create_plane_mesh(map_configs),
                texture: texture_from_color_map(&map_data.color_map),
                vertical_offset: -0.01,
                show_wireframe: false,
            }
        }
        DrawMode::Mesh => {
            RenderAssets {
                mesh: MeshGenerator::generate_terrain_mesh(
                    &map_data.noise_map,
                    map_configs.height_multiplier,
                    &map_configs.height_curve,
                    map_configs.level_of_detail,
                )
                .create_mesh(),
                texture: texture_from_color_map(&map_data.color_map),
                vertical_offset: 0.0,
                show_wireframe: map_configs.show_uv_wireframe,
            }
        }
        DrawMode::EndlessTerrain => RenderAssets {
            mesh: create_plane_mesh(map_configs),
            texture: texture_from_height_map(&map_data.noise_map),
            vertical_offset: -0.01,
            show_wireframe: false,
        },
        DrawMode::FallOffMap => {
            let fall_off_map =
                FallOffGenerator::generate_fall_off_map(map_configs.map_chunk_size as usize);
            RenderAssets {
                mesh: create_plane_mesh(map_configs),
                texture: texture_from_height_map(&fall_off_map),
                vertical_offset: -0.01,
                show_wireframe: false,
            }
        }
    }
}

fn create_plane_mesh(map_configs: &MapConfigs) -> Mesh {
    Plane3d::default()
        .mesh()
        .size(
            map_configs.map_chunk_size as f32,
            map_configs.map_chunk_size as f32,
        )
        .into()
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

pub(crate) fn chunk_span(map_configs: &MapConfigs) -> f32 {
    map_configs.map_chunk_size.saturating_sub(1).max(1) as f32
}

pub(crate) fn generate_map_data(map_configs: &MapConfigs, coord: IVec2) -> MapData {
    let mut noise_map = generate_noise_map_for_chunk(map_configs, coord);
    let size = map_configs.map_chunk_size as usize;
    let mut color_map = vec![vec![]; size];
    for y in 0..size {
        for x in 0..size {
            if map_configs.use_falloff_map {
                noise_map[y][x] = (noise_map[y][x] - map_configs.falloff_map[y][x]).clamp(0.0, 1.0);
            }
            let height_value = noise_map[y][x];
            let color = terrain_color_for_height(height_value, map_configs.regions.as_slice());
            color_map[y].push(color);
        }
    }
    MapData::new(noise_map, color_map)
}

pub(crate) fn generate_noise_map_for_chunk(
    map_configs: &MapConfigs,
    chunk_coord: IVec2,
) -> Vec<Vec<f32>> {
    let width = map_configs.map_chunk_size as usize;
    let height = map_configs.map_chunk_size as usize;
    let scale = map_configs.scale.max(0.0001);
    let octaves = map_configs.octaves.max(1) as usize;
    let half_width = map_configs.map_chunk_size as f64 / 2.0;
    let half_height = map_configs.map_chunk_size as f64 / 2.0;
    let perlin = Perlin::new(0);
    let octave_offsets = build_octave_offsets(map_configs, octaves);
    let chunk_span = map_configs.map_chunk_size.saturating_sub(1) as f64;
    let chunk_offset_x = chunk_coord.x as f64 * chunk_span;
    let chunk_offset_y = chunk_coord.y as f64 * chunk_span;
    let max_possible_height = max_possible_noise_height(map_configs, octaves);

    let mut noise_map = vec![vec![0.0_f32; width]; height];

    for y in 0..height {
        for x in 0..width {
            let mut amplitude = 1.0;
            let mut frequency = map_configs.frequency.max(0.0001);
            let mut noise_height = 0.0;

            for &(offset_x, offset_y) in &octave_offsets {
                let sample_x =
                    ((chunk_offset_x + x as f64 - half_width) / scale) * frequency + offset_x;
                let sample_y =
                    ((chunk_offset_y - y as f64 + half_height) / scale) * frequency + offset_y;

                let perlin_value = perlin.get([sample_x, sample_y]);
                noise_height += perlin_value * amplitude;

                amplitude *= map_configs.persistence;
                frequency *= map_configs.lacunarity;
            }

            noise_map[y][x] = if max_possible_height <= f64::EPSILON {
                0.0
            } else {
                (((noise_height + max_possible_height) / (max_possible_height * 2.0))
                    .clamp(0.0, 1.0)) as f32
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

fn max_possible_noise_height(map_configs: &MapConfigs, octaves: usize) -> f64 {
    let mut amplitude = 1.0;
    let mut max_possible_height = 0.0;

    for _ in 0..octaves {
        max_possible_height += amplitude;
        amplitude *= map_configs.persistence;
    }

    max_possible_height
}

fn setup_noise_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut map_configs: ResMut<Persistent<MapConfigs>>,
) {
    if map_configs.draw_mode == DrawMode::EndlessTerrain {
        return;
    }
    let map_configs_b = map_configs.sanitized();
    let render_assets = create_render_assets(&map_configs);
    let texture_handle = images.add(render_assets.texture);
    let mesh_handle = meshes.add(render_assets.mesh);

    let mut entity_commands = commands.spawn((
        GroundPlane,
        Name::new("GroundPlane"),
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            perceptual_roughness: 1.0,
            ..default()
        })),
        Transform::from_xyz(0.0, render_assets.vertical_offset, 0.0),
    ));
    map_configs.set_falloff_map(fall_off_generator::FallOffGenerator::generate_fall_off_map(
        map_configs_b.map_chunk_size as usize,
    ));
    apply_wireframe_debug(&mut entity_commands, render_assets.show_wireframe);
}

fn refresh_noise_plane(
    mut commands: Commands,
    map_configs: Res<Persistent<MapConfigs>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    ground_plane_entities: Query<Entity, With<GroundPlane>>,
    mut ground_plane_query: Query<
        (
            Entity,
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Transform,
        ),
        With<GroundPlane>,
    >,
) {
    if map_configs.draw_mode == DrawMode::EndlessTerrain {
        for entity in &ground_plane_entities {
            commands.entity(entity).despawn();
        }
        return;
    }

    let render_assets = create_render_assets(&map_configs);

    let Ok((entity, mut mesh_handle, mut material_handle, mut transform)) =
        ground_plane_query.single_mut()
    else {
        let texture_handle = images.add(render_assets.texture);
        let mesh_handle = meshes.add(render_assets.mesh);
        let mut entity_commands = commands.spawn((
            GroundPlane,
            Name::new("GroundPlane"),
            Mesh3d(mesh_handle),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(texture_handle),
                perceptual_roughness: 1.0,
                ..default()
            })),
            Transform::from_xyz(0.0, render_assets.vertical_offset, 0.0),
        ));

        apply_wireframe_debug(&mut entity_commands, render_assets.show_wireframe);
        return;
    };

    *mesh_handle = Mesh3d(meshes.add(render_assets.mesh));

    *material_handle = MeshMaterial3d(materials.add(StandardMaterial {
        base_color_texture: Some(images.add(render_assets.texture)),
        perceptual_roughness: 1.0,
        ..default()
    }));

    transform.translation = Vec3::new(0.0, render_assets.vertical_offset, 0.0);

    let mut entity_commands = commands.entity(entity);
    apply_wireframe_debug(&mut entity_commands, render_assets.show_wireframe);
}

fn apply_wireframe_debug(entity_commands: &mut EntityCommands, show_wireframe: bool) {
    if show_wireframe {
        entity_commands.insert((
            Wireframe,
            WireframeColor {
                color: MESH_DEBUG_WIREFRAME_COLOR.into(),
            },
        ));
    } else {
        entity_commands.remove::<Wireframe>();
        entity_commands.remove::<WireframeColor>();
    }
}

pub struct MapData {
    pub noise_map: Vec<Vec<f32>>,
    pub color_map: Vec<Vec<[u8; 4]>>,
}

impl MapData {
    pub fn new(noise_map: Vec<Vec<f32>>, color_map: Vec<Vec<[u8; 4]>>) -> Self {
        Self {
            noise_map,
            color_map,
        }
    }
}
