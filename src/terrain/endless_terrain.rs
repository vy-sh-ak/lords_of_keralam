use std::collections::HashMap;

use bevy::{
    pbr::wireframe::{Wireframe, WireframeColor}, prelude::*
};
use bevy_persistent::Persistent;

use crate::{
    camera_config::{CameraSettings, CameraSystems},
    terrain::{
        MapGenerator, TerrainSampler, generate_map_data,
        terrain_material::{TerrainMaterial, build_terrain_material},
    },
};

use super::{
    DrawMode, EndlessTerrainLodBand, MeshGenerator, chunk_span,
};

pub struct EndlessTerrainPlugin;

const CHUNK_BUILD_BUDGET_PER_FRAME: usize = 2;
const ENDLESS_TERRAIN_WIREFRAME_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
const MIN_VISIBLE_CHUNK_MARGIN: f32 = 1.5;

#[derive(Component)]
struct EndlessTerrainChunk {
    level_of_detail: u32,
    terrain_epoch: u64,
}

struct DesiredChunk {
    coord: IVec2,
    level_of_detail: u32,
    edge_distance: f32,
}

#[derive(Resource)]
struct EndlessTerrainState {
    active_chunks: HashMap<IVec2, Entity>,
    terrain_epoch: u64,
    material_handle: Option<Handle<TerrainMaterial>>,
}

impl Default for EndlessTerrainState {
    fn default() -> Self {
        Self {
            active_chunks: Default::default(),
            terrain_epoch: Default::default(),
            material_handle: None,
        }
    }
}

impl Plugin for EndlessTerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EndlessTerrainState::default())
            .add_systems(
                Update,
                (
                    sync_endless_terrain.after(CameraSystems::UpdateState),
                    update_focus_height,
                ),
            );
    }
}

fn update_focus_height(
    terrain_sampler: Res<TerrainSampler>,
    map_generator: Res<Persistent<MapGenerator>>,
    mut camera_settings: ResMut<CameraSettings>,
    time: Res<Time>,
) {
   if camera_settings.zoom < 0.5 {
        return;
    }
    let forward = Vec2::new(
        -camera_settings.orbit_yaw.cos(),
        -camera_settings.orbit_yaw.sin(),
    );
    let mut highest = f32::MIN;

for distance in [5.0, 10.0, 15.0, 20.0] {
    let pos =
        camera_settings.focus_xz
        + forward * distance;

    let h =
        terrain_sampler.sample_height(
            &map_generator,
            pos.x,
            pos.y,
        );

    highest = highest.max(h);
}

    let desired_clearance = 6.0;
    let target_height = highest + desired_clearance;

    if target_height > camera_settings.focus_height {
        // climb fast
        camera_settings.focus_height = camera_settings
            .focus_height
            .lerp(target_height, time.delta_secs() * 15.0);
    } else {
        // descend slowly
        camera_settings.focus_height = camera_settings
            .focus_height
            .lerp(target_height, time.delta_secs() * 2.0);
    }
}

fn sync_endless_terrain(
    mut commands: Commands,
    camera_settings: Res<CameraSettings>,
    camera_transform: Single<&Transform, With<Camera>>,
    mut state: ResMut<EndlessTerrainState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    asset_server: Res<AssetServer>,
    mut chunk_query: Query<(
        Entity,
        &EndlessTerrainChunk,
        &mut Mesh3d,
        &mut MeshMaterial3d<TerrainMaterial>,
    )>,
    terrain_sampler: Res<TerrainSampler>,
    map_generator: Res<Persistent<MapGenerator>>,
) {
    if map_generator.draw_mode != DrawMode::EndlessTerrain {
        for (entity, _, _, _) in &mut chunk_query {
            commands.entity(entity).despawn();
        }
        state.active_chunks.clear();
        return;
    }

    if map_generator.is_changed() {
        state.terrain_epoch = state.terrain_epoch.saturating_add(1);
        state.material_handle = None;
    }

    let chunk_span = chunk_span(&map_generator);
    let camera_position = camera_transform.translation;
    let viewer_position = camera_settings.focus_xz;
    let viewer_chunk = world_to_chunk_coord(viewer_position, chunk_span);
    let focus = Vec3::new(
        camera_settings.focus_xz.x,
        camera_settings.focus_height,
        camera_settings.focus_xz.y,
    );
    let view_radius = visible_radius_from_camera(camera_position, focus, chunk_span);
    let max_visible_distance = map_generator.max_endless_visible_distance().min(view_radius);
    let chunk_radius = ((max_visible_distance / chunk_span).ceil() as i32).max(1) + 1;
    let desired_chunks = collect_desired_chunks(
        viewer_position,
        viewer_chunk,
        chunk_radius,
        chunk_span,
        max_visible_distance,
        &map_generator.endless_lod_bands,
    );
    let desired_coords: Vec<_> = desired_chunks.iter().map(|chunk| chunk.coord).collect();

    // Build shared terrain material (cached after first creation)
    let material_handle = if let Some(ref handle) = state.material_handle {
        handle.clone()
    } else {
        let handle = build_terrain_material(
            &mut terrain_materials,
            &asset_server,
            &map_generator,
        );
        state.material_handle = Some(handle.clone());
        handle
    };

    let stale_coords: Vec<_> = state
        .active_chunks
        .keys()
        .copied()
        .filter(|coord| !desired_coords.contains(coord))
        .collect();

    for coord in stale_coords {
        if let Some(entity) = state.active_chunks.remove(&coord) {
            commands.entity(entity).despawn();
        }
    }

    let mut remaining_build_budget = CHUNK_BUILD_BUDGET_PER_FRAME;

    for desired_chunk in desired_chunks {
        let coord = desired_chunk.coord;
        let lod = desired_chunk.level_of_detail;

        if let Some(entity) = state.active_chunks.get(&coord).copied() {
            let Ok((_, chunk, mut mesh_handle, mut material_comp)) = chunk_query.get_mut(entity)
            else {
                state.active_chunks.remove(&coord);
                continue;
            };

            let needs_rebuild =
                chunk.level_of_detail != lod || chunk.terrain_epoch != state.terrain_epoch;
            if needs_rebuild && remaining_build_budget > 0 {
                let mesh =
                    build_chunk_mesh(coord, lod, terrain_sampler.as_ref(), &map_generator);
                *mesh_handle = Mesh3d(meshes.add(mesh));
                *material_comp = MeshMaterial3d(material_handle.clone());
                commands.entity(entity).insert(EndlessTerrainChunk {
                    level_of_detail: lod,
                    terrain_epoch: state.terrain_epoch,
                });
                remaining_build_budget -= 1;
            }

            let mut entity_commands = commands.entity(entity);
            apply_wireframe_debug(&mut entity_commands, map_generator.show_uv_wireframe);
        } else {
            if remaining_build_budget == 0 {
                continue;
            }

            let mesh =
                build_chunk_mesh(coord, lod, terrain_sampler.as_ref(), &map_generator);
            let mut entity_commands = commands.spawn((
                EndlessTerrainChunk {
                    level_of_detail: lod,
                    terrain_epoch: state.terrain_epoch,
                },
                Name::new(format!("EndlessTerrainChunk({}, {})", coord.x, coord.y)),
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material_handle.clone()),
                Transform::from_xyz(
                    coord.x as f32 * chunk_span,
                    0.0,
                    coord.y as f32 * chunk_span,
                ),
            ));
            apply_wireframe_debug(&mut entity_commands, map_generator.show_uv_wireframe);
            let entity = entity_commands.id();
            state.active_chunks.insert(coord, entity);
            remaining_build_budget -= 1;
        }
    }
}

fn collect_desired_chunks(
    viewer_position: Vec2,
    viewer_chunk: IVec2,
    chunk_radius: i32,
    chunk_span: f32,
    max_visible_distance: f32,
    lod_bands: &[EndlessTerrainLodBand],
) -> Vec<DesiredChunk> {
    let mut desired_chunks = Vec::new();
    let chunk_half_extent = chunk_span * 0.5;

    for y_offset in -chunk_radius..=chunk_radius {
        for x_offset in -chunk_radius..=chunk_radius {
            let coord = viewer_chunk + IVec2::new(x_offset, y_offset);
            let center = chunk_center(coord, chunk_span);
            let dx = (viewer_position.x - center.x).abs() - chunk_half_extent;
            let dz = (viewer_position.y - center.y).abs() - chunk_half_extent;
            let edge_distance = dx.max(0.0).hypot(dz.max(0.0));

            if edge_distance > max_visible_distance {
                continue;
            }

            let level_of_detail = select_level_of_detail(edge_distance, lod_bands);
            desired_chunks.push(DesiredChunk {
                coord,
                level_of_detail,
                edge_distance,
            });
        }
    }

    desired_chunks.sort_by(|left, right| left.edge_distance.total_cmp(&right.edge_distance));

    desired_chunks
}

fn build_chunk_mesh(
    coord: IVec2,
    level_of_detail: u32,
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
) -> Mesh {
    let map_data = generate_map_data(terrain_sampler, coord, map_generator);
    MeshGenerator::generate_terrain_mesh(
        &map_data.noise_map,
        map_generator.terrain_data.height_multiplier,
        &map_generator.terrain_data.height_curve,
        level_of_detail,
    )
    .create_mesh()
}

fn world_to_chunk_coord(world_position: Vec2, chunk_span: f32) -> IVec2 {
    IVec2::new(
        (world_position.x / chunk_span).round() as i32,
        (world_position.y / chunk_span).round() as i32,
    )
}

fn chunk_center(coord: IVec2, chunk_span: f32) -> Vec2 {
    Vec2::new(coord.x as f32 * chunk_span, coord.y as f32 * chunk_span)
}

fn select_level_of_detail(distance: f32, lod_bands: &[EndlessTerrainLodBand]) -> u32 {
    lod_bands
        .iter()
        .find(|band| distance <= band.visible_distance)
        .or_else(|| lod_bands.last())
        .map(|band| band.level_of_detail)
        .unwrap_or(0)
        .min(2)
}

fn visible_radius_from_camera(camera_position: Vec3, focus: Vec3, chunk_span: f32) -> f32 {
    let focus_distance = camera_position.distance(focus);
    let forward = (focus - camera_position).normalize();
    let sin_pitch = (-forward.y).max(0.01); // 1 = looking straight down, ~0 = looking horizontal

    let view_radius = focus_distance * 2.0 / sin_pitch;

    view_radius.max(chunk_span * MIN_VISIBLE_CHUNK_MARGIN)
}

fn apply_wireframe_debug(entity_commands: &mut EntityCommands, show_wireframe: bool) {
    if show_wireframe {
        entity_commands.insert((
            Wireframe,
            WireframeColor {
                color: ENDLESS_TERRAIN_WIREFRAME_COLOR.into(),
            },
        ));
    } else {
        entity_commands.remove::<Wireframe>();
        entity_commands.remove::<WireframeColor>();
    }
}
