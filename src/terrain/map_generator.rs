use super::{MeshGenerator, texture_generator::texture_from_height_map};
use crate::{
    editor_config::{AutosaveAppExt, EditorStateAppExt},
    persistence,
    terrain::{
        FallOffGenerator, NoiseData, TerrainData, TerrainSampler, TextureData,
        terrain_material::{TerrainMaterial, build_terrain_material},
    },
};
use bevy::{
    pbr::wireframe::{Wireframe, WireframeColor},
    prelude::*,
};
use bevy_inspector_egui::prelude::*;
use bevy_persistent::prelude::*;
use serde::{Deserialize, Serialize};
const MESH_DEBUG_WIREFRAME_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);

#[derive(Reflect, InspectorOptions, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[reflect(InspectorOptions)]
pub enum DrawMode {
    NoiseMap,
    Mesh,
    EndlessTerrain,
    FallOffMap,
}
impl DrawMode {
    pub const ALL: [Self; 4] = [
        Self::NoiseMap,
        Self::Mesh,
        Self::EndlessTerrain,
        Self::FallOffMap,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::NoiseMap => "Noise Map",
            Self::Mesh => "Mesh",
            Self::EndlessTerrain => "Endless Terrain",
            Self::FallOffMap => "Fall-Off Map",
        }
    }
}

impl Default for DrawMode {
    fn default() -> Self {
        Self::NoiseMap
    }
}

#[derive(Reflect, InspectorOptions, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[reflect(InspectorOptions)]
pub struct EndlessTerrainLodBand {
    #[inspector(min = 0, max = 6)]
    pub level_of_detail: u32,
    #[inspector(min = 0.0, max = 10000.0)]
    pub visible_distance: f32,
}

impl EndlessTerrainLodBand {
    fn new(level_of_detail: u32, visible_distance: f32) -> Self {
        Self {
            level_of_detail,
            visible_distance,
        }
    }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[reflect(Resource, InspectorOptions)]
pub struct MapGenerator {
    #[reflect(ignore)]
    pub map_chunk_size: u32,
    #[serde(skip)]
    #[reflect(ignore)]
    pub falloff_map: Vec<Vec<f32>>,

    #[inspector(min = 0, max = 6)]
    pub level_of_detail: u32,

    pub draw_mode: DrawMode,
    #[serde(default)]
    pub show_uv_wireframe: bool,
    #[serde(default = "default_endless_lod_bands")]
    pub endless_lod_bands: Vec<EndlessTerrainLodBand>,

    pub noise_data: NoiseData,
    pub terrain_data: TerrainData,
    pub texture_data: TextureData,
}

impl Default for MapGenerator {
    fn default() -> Self {
        Self {
            map_chunk_size: 241,
            level_of_detail: 0,
            draw_mode: DrawMode::default(),
            show_uv_wireframe: false,
            endless_lod_bands: default_endless_lod_bands(),
            falloff_map: vec![],
            noise_data: NoiseData {
                frequency: 0.3,
                scale: 10.0,
                octaves: 4,
                lacunarity: 2.0,
                persistence: 0.5,
                offset_x: 0.0,
                offset_y: 0.0,
                seed: 0,
            },
            terrain_data: TerrainData {
                use_falloff_map: false,
                height_multiplier: 20.0,
                height_curve: crate::terrain::HeightCurve::default(),
            },
            texture_data: TextureData {
                min_height: -10.0,
                max_height: 40.0,
                layers: vec![
                    crate::terrain::data::TextureLayerConfig {
                        texture_path: "textures/water.png".to_string(),
                        start_height: 0.2,
                        blend_strength: 0.1,
                        tint_strength: 0.0,
                        texture_scale: 5.0,
                        tint: LinearRgba::new(0.0, 0.0, 0.8, 1.0),
                    },
                    crate::terrain::data::TextureLayerConfig {
                        texture_path: "textures/sand.png".to_string(),
                        start_height: 0.3,
                        blend_strength: 0.1,
                        tint_strength: 0.0,
                        texture_scale: 8.0,
                        tint: LinearRgba::new(0.9, 0.85, 0.5, 1.0),
                    },
                    crate::terrain::data::TextureLayerConfig {
                        texture_path: "textures/grass.png".to_string(),
                        start_height: 0.35,
                        blend_strength: 0.15,
                        tint_strength: 0.0,
                        texture_scale: 12.0,
                        tint: LinearRgba::new(0.2, 0.7, 0.1, 1.0),
                    },
                    crate::terrain::data::TextureLayerConfig {
                        texture_path: "textures/rock.png".to_string(),
                        start_height: 0.8,
                        blend_strength: 0.1,
                        tint_strength: 0.0,
                        texture_scale: 20.0,
                        tint: LinearRgba::new(0.5, 0.5, 0.5, 1.0),
                    },
                ],
            },
        }
    }
}

fn default_endless_lod_bands() -> Vec<EndlessTerrainLodBand> {
    vec![
        EndlessTerrainLodBand::new(0, 220.0),
        EndlessTerrainLodBand::new(2, 420.0),
        EndlessTerrainLodBand::new(4, 700.0),
        EndlessTerrainLodBand::new(6, 1050.0),
    ]
}

impl MapGenerator {
    const MIN_VISIBLE_DISTANCE: f32 = 1.0;
    pub fn plugin(self) -> MapGeneratorPlugin {
        MapGeneratorPlugin
    }
    fn set_falloff_map(&mut self, falloff_map: Vec<Vec<f32>>) -> bool {
        if self.falloff_map == falloff_map {
            return false;
        }

        self.falloff_map = falloff_map;
        true
    }
    pub fn max_endless_visible_distance(&self) -> f32 {
        self.endless_lod_bands
            .last()
            .map(|band| band.visible_distance.max(Self::MIN_VISIBLE_DISTANCE))
            .unwrap_or(Self::MIN_VISIBLE_DISTANCE)
    }
}
pub struct MapGeneratorPlugin;

struct RenderAssets {
    mesh: Mesh,
    texture: Option<Image>,
    vertical_offset: f32,
    show_wireframe: bool,
}

#[derive(Component)]
struct GroundPlane;

impl Plugin for MapGeneratorPlugin {
    fn build(&self, app: &mut App) {
        let generator_persistence = persistence::PersistenceConfig::new("map_generator");
        let map_generator = generator_persistence
            .get_resource::<MapGenerator>("map generator", "map_generator.toml");

        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default())
            .register_type::<NoiseData>()
            .register_type::<TerrainData>()
            .register_type::<TextureData>()
            .register_type::<MapGenerator>()
            .insert_resource(map_generator)
            .add_editor_state::<MapGenerator>()
            .add_autosave::<MapGenerator>()
            .insert_resource(TerrainSampler::default())
            .add_systems(Startup, setup_noise_plane)
            .add_systems(
                Update,
                refresh_noise_plane.run_if(resource_changed::<Persistent<MapGenerator>>),
            );
    }
}

fn spawn_ground_plane(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    standard_materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    asset_server: &AssetServer,
    terrain_materials: &mut Assets<TerrainMaterial>,
    render_assets: &RenderAssets,
    map_generator: &MapGenerator,
) {
    let mesh_handle = meshes.add(render_assets.mesh.clone());
    let mut entity_commands = commands.spawn((
        GroundPlane,
        Name::new("GroundPlane"),
        Mesh3d(mesh_handle),
        Transform::from_xyz(0.0, render_assets.vertical_offset, 0.0),
    ));

    match map_generator.draw_mode {
        DrawMode::Mesh => {
            let mat_handle = build_terrain_material(terrain_materials, asset_server, map_generator);
            info!(
                "Spawned GroundPlane with TerrainMaterial handle={:?}",
                mat_handle
            );
            entity_commands.insert(MeshMaterial3d(mat_handle));
        }
        DrawMode::NoiseMap | DrawMode::FallOffMap => {
            if let Some(texture) = &render_assets.texture {
                let texture_handle = images.add(texture.clone());
                entity_commands.insert(MeshMaterial3d(standard_materials.add(StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    perceptual_roughness: 1.0,
                    ..default()
                })));
            }
        }
        _ => {}
    }

    apply_wireframe_debug(&mut entity_commands, render_assets.show_wireframe);
}

fn create_render_assets(
    map_generator: &MapGenerator,
    terrain_sampler: &TerrainSampler,
) -> RenderAssets {
    let map_data = generate_map_data(terrain_sampler, IVec2::ZERO, map_generator);
    info!(
        "Generated noise map for render assets {draw_mode:?}",
        draw_mode = map_generator.draw_mode
    );
    match map_generator.draw_mode {
        DrawMode::NoiseMap => RenderAssets {
            mesh: create_plane_mesh(map_generator),
            texture: Some(texture_from_height_map(&map_data.noise_map)),
            vertical_offset: -0.01,
            show_wireframe: map_generator.show_uv_wireframe,
        },
        DrawMode::Mesh => RenderAssets {
            mesh: MeshGenerator::generate_terrain_mesh(
                &map_data.noise_map,
                map_generator.terrain_data.height_multiplier,
                &map_generator.terrain_data.height_curve,
                map_generator.level_of_detail,
            )
            .create_mesh(),
            texture: None,
            vertical_offset: 0.0,
            show_wireframe: map_generator.show_uv_wireframe,
        },
        DrawMode::EndlessTerrain => RenderAssets {
            mesh: create_plane_mesh(map_generator),
            texture: Some(texture_from_height_map(&map_data.noise_map)),
            vertical_offset: -0.01,
            show_wireframe: map_generator.show_uv_wireframe,
        },
        DrawMode::FallOffMap => {
            let fall_off_map =
                FallOffGenerator::generate_fall_off_map(map_generator.map_chunk_size as usize);
            RenderAssets {
                mesh: create_plane_mesh(map_generator),
                texture: Some(texture_from_height_map(&fall_off_map)),
                vertical_offset: -0.01,
                show_wireframe: map_generator.show_uv_wireframe,
            }
        }
    }
}

fn create_plane_mesh(map_generator: &MapGenerator) -> Mesh {
    Plane3d::default()
        .mesh()
        .size(
            map_generator.map_chunk_size as f32,
            map_generator.map_chunk_size as f32,
        )
        .into()
}

pub(crate) fn chunk_span(map_generator: &MapGenerator) -> f32 {
    map_generator.map_chunk_size.saturating_sub(1).max(1) as f32
}

pub(crate) fn ensure_falloff_map(map_generator: &mut MapGenerator) {
    if map_generator.terrain_data.use_falloff_map && map_generator.falloff_map.is_empty() {
        let map_size = map_generator.map_chunk_size as usize + 2;
        map_generator.set_falloff_map(FallOffGenerator::generate_fall_off_map(map_size));
    }
}

pub(crate) fn generate_map_data(
    terrain_sampler: &TerrainSampler,
    coord: IVec2,
    map_generator: &MapGenerator,
) -> MapData {
    let mut noise_map = generate_noise_map_for_chunk(terrain_sampler, coord, map_generator);

    if map_generator.terrain_data.use_falloff_map {
        for y in 0..noise_map.len() {
            for x in 0..noise_map[y].len() {
                noise_map[y][x] =
                    (noise_map[y][x] - map_generator.falloff_map[y][x]).clamp(0.0, 1.0);
            }
        }
    }

    MapData::new(noise_map)
}

pub(crate) fn generate_noise_map_for_chunk(
    terrain_sampler: &TerrainSampler,
    chunk_coord: IVec2,
    map_generator: &MapGenerator,
) -> Vec<Vec<f32>> {
    let width = map_generator.map_chunk_size as usize + 2;
    let height = map_generator.map_chunk_size as usize + 2;
    let half_width = map_generator.map_chunk_size as f64 / 2.0;
    let half_height = map_generator.map_chunk_size as f64 / 2.0;
    let chunk_span = map_generator.map_chunk_size.saturating_sub(1) as f64;
    let chunk_offset_x = chunk_coord.x as f64 * chunk_span;
    let chunk_offset_y = chunk_coord.y as f64 * chunk_span;

    let mut noise_map = vec![vec![0.0_f32; width]; height];

    for y in 0..height {
        for x in 0..width {
            let world_x = chunk_offset_x + x as f64 - half_width;
            let world_z = chunk_offset_y - y as f64 + half_height;
            noise_map[y][x] = terrain_sampler.sample_noise(map_generator, world_x, world_z);
        }
    }

    noise_map
}

fn setup_noise_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    terrain_sampler: Res<TerrainSampler>,
    mut map_generator: ResMut<Persistent<MapGenerator>>,
) {
    ensure_falloff_map(&mut map_generator);
    if map_generator.draw_mode == DrawMode::EndlessTerrain {
        return;
    }
    let render_assets = create_render_assets(&map_generator, terrain_sampler.as_ref());
    spawn_ground_plane(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        &mut images,
        &asset_server,
        &mut terrain_materials,
        &render_assets,
        &map_generator,
    );
}

fn refresh_noise_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    ground_plane_entities: Query<Entity, With<GroundPlane>>,
    map_generator: Res<Persistent<MapGenerator>>,
    mut ground_plane_query: Query<(Entity, &mut Mesh3d, &mut Transform), With<GroundPlane>>,
    terrain_sampler: Res<TerrainSampler>,
) {
    if map_generator.draw_mode == DrawMode::EndlessTerrain {
        for entity in &ground_plane_entities {
            commands.entity(entity).despawn();
        }
        return;
    }

    let render_assets = create_render_assets(&map_generator, terrain_sampler.as_ref());

    let entity = match ground_plane_query.single_mut() {
        Ok((entity, mut mesh_handle, mut transform)) => {
            *mesh_handle = Mesh3d(meshes.add(render_assets.mesh.clone()));
            transform.translation = Vec3::new(0.0, render_assets.vertical_offset, 0.0);
            entity
        }
        Err(_) => {
            spawn_ground_plane(
                &mut commands,
                &mut meshes,
                &mut standard_materials,
                &mut images,
                &asset_server,
                &mut terrain_materials,
                &render_assets,
                &map_generator,
            );
            return;
        }
    };

    match map_generator.draw_mode {
        DrawMode::Mesh => {
            commands
                .entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>();
            let mat_handle =
                build_terrain_material(&mut terrain_materials, &asset_server, &map_generator);
            commands.entity(entity).insert(MeshMaterial3d(mat_handle));
        }
        _ => {
            commands
                .entity(entity)
                .remove::<MeshMaterial3d<TerrainMaterial>>();
            if let Some(texture) = &render_assets.texture {
                let texture_handle = images.add(texture.clone());
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(standard_materials.add(StandardMaterial {
                        base_color_texture: Some(texture_handle),
                        perceptual_roughness: 1.0,
                        ..default()
                    })));
            }
        }
    }

    apply_wireframe_debug(&mut commands.entity(entity), render_assets.show_wireframe);
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
}

impl MapData {
    pub fn new(noise_map: Vec<Vec<f32>>) -> Self {
        Self { noise_map }
    }
}
