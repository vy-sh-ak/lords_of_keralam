pub mod brush;
pub mod ui;

use std::collections::HashMap;

use bevy::prelude::*;

use bevy_persistent::Persistent;

use crate::terrain::endless_terrain::{sync_endless_terrain, EndlessTerrainState};
use crate::terrain::{MapGenerator, TerrainSampler};
use crate::ui_editor::UIKeyboardCapture;
use crate::world_grid_config::WorldGrid;

#[derive(Resource, Default)]
pub struct SculptMap {
    pub chunks: HashMap<IVec2, Vec<Vec<f32>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainTool {
    Raise,
    Lower,
    Flatten,
    Smooth,
}

#[derive(Resource)]
pub struct BrushConfig {
    pub active: bool,
    pub tool: TerrainTool,
    pub radius: f32,
    pub strength: f32,
    pub flatten_target: Option<f32>,
    pub flatten_sampling: bool,
}

impl Default for BrushConfig {
    fn default() -> Self {
        Self {
            active: false,
            tool: TerrainTool::Raise,
            radius: 5.0,
            strength: 1.0,
            flatten_target: None,
            flatten_sampling: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct SyncGridRequest(pub bool);

pub struct TerrainPainterPlugin;

impl Plugin for TerrainPainterPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SculptMap::default())
            .insert_resource(BrushConfig::default())
            .insert_resource(SyncGridRequest::default())
            .add_systems(Update, sculpt_paint_system.before(sync_endless_terrain))
            .add_systems(Update, draw_custom_cursor)
            .add_systems(Update, sync_grid_system);
    }
}

fn sync_grid_system(
    mut request: ResMut<SyncGridRequest>,
    mut world_grid: ResMut<WorldGrid>,
    terrain_sampler: Res<TerrainSampler>,
    map_generator: Res<Persistent<MapGenerator>>,
    sculpt_map: Res<SculptMap>,
) {
    if !request.0 {
        return;
    }
    request.0 = false;
    sync_grid_heights(&mut world_grid, &terrain_sampler, &map_generator, &sculpt_map);
}

fn sculpt_paint_system(
    mut sculpt_map: ResMut<SculptMap>,
    mut brush_config: ResMut<BrushConfig>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    terrain_sampler: Res<TerrainSampler>,
    map_generator: Res<Persistent<MapGenerator>>,
    time: Res<Time>,
    keyboard_capture: Option<Res<UIKeyboardCapture>>,
    mut endless_state: ResMut<EndlessTerrainState>,
) {
    if !brush_config.active {
        return;
    }

    if let Some(capture) = keyboard_capture.as_ref() {
        if capture.wants_pointer_input || capture.is_typing {
            return;
        }
    }

    // Handle flatten target sampling
    if brush_config.flatten_sampling {
        if mouse_buttons.just_pressed(MouseButton::Left) {
            let (camera, camera_transform) = camera_query.into_inner();
            if let Some(hit_pos) =
                get_terrain_hit_position(camera, camera_transform, &window, &terrain_sampler, &map_generator)
            {
                brush_config.flatten_target = Some(hit_pos.y);
            }
            brush_config.flatten_sampling = false;
        }
        return;
    }

    let is_left = mouse_buttons.pressed(MouseButton::Left);
    let is_right = mouse_buttons.pressed(MouseButton::Right);

    if !is_left && !is_right {
        return;
    }

    let invert = is_right;
    // For Flatten/Smooth, both buttons paint (no invert)
    let invert = match brush_config.tool {
        TerrainTool::Raise | TerrainTool::Lower => invert,
        TerrainTool::Flatten | TerrainTool::Smooth => false,
    };

    let (camera, camera_transform) = camera_query.into_inner();
    let Some(hit_pos) =
        get_terrain_hit_position(camera, camera_transform, &window, &terrain_sampler, &map_generator)
    else {
        return;
    };

    let dt = time.delta_secs();
    let affected = brush::apply_brush(
        &mut sculpt_map.chunks,
        hit_pos,
        &brush_config,
        &terrain_sampler,
        &map_generator,
        dt,
        invert,
    );

    for coord in affected {
        endless_state.mark_chunk_dirty(coord);
    }
}

fn get_terrain_hit_position(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
) -> Option<Vec3> {
    let cursor_pos = window.cursor_position()?;
    let ray = camera.viewport_to_world(camera_transform, cursor_pos).ok()?;
    let distance = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Dir3::Y))?;
    let plane_pos = ray.get_point(distance);
    let height = terrain_sampler.sample_height(map_generator, plane_pos.x, plane_pos.z);
    Some(Vec3::new(plane_pos.x, height, plane_pos.z))
}

fn get_terrain_hit_position_with_sculpt(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
    sculpt_map: &SculptMap,
) -> Option<Vec3> {
    let cursor_pos = window.cursor_position()?;
    let ray = camera.viewport_to_world(camera_transform, cursor_pos).ok()?;
    let distance = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Dir3::Y))?;
    let plane_pos = ray.get_point(distance);
    let height = sample_total_height(
        terrain_sampler,
        map_generator,
        sculpt_map,
        Vec3::new(plane_pos.x, 0.0, plane_pos.z),
    );
    Some(Vec3::new(plane_pos.x, height, plane_pos.z))
}

fn vertex_total_height(
    nx: i64,
    nz: i64,
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
    sculpt_map: &SculptMap,
    chunk_span_val: f32,
    half: f32,
) -> f32 {
    // Mesh vertex world positions are at n - 0.5 for integer n
    // (because odd map_chunk_size (241) gives half = 120.5)
    let span = chunk_span_val as i64;
    let half_shift = (half - 0.5) as i64; // 120

    // X: vertex index increases with world_x
    let shifted_x = nx + half_shift;
    let cx = shifted_x.div_euclid(span) as i32;
    let ix = (shifted_x - cx as i64 * span) as usize;

    // Z: vertex index DECREASES as world_z increases (mesh z = half - index)
    let shifted_z = nz + half_shift;
    let cz = shifted_z.div_euclid(span) as i32;
    let iz = (cz as i64 * span + (2 * half_shift as i64 + 1) - shifted_z) as usize;

    let vertex_x = cx as f32 * chunk_span_val + ix as f32 - half;
    let vertex_z = cz as f32 * chunk_span_val + half - iz as f32;

    let base = terrain_sampler.sample_height(map_generator, vertex_x, vertex_z);

    if let Some(chunk) = sculpt_map.chunks.get(&IVec2::new(cx, cz)) {
        if iz < chunk.len() && ix < chunk[0].len() {
            return base + chunk[iz][ix];
        }
    }
    base
}

pub(crate) fn sample_total_height(
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
    sculpt_map: &SculptMap,
    world_pos: Vec3,
) -> f32 {
    let chunk_span_val = crate::terrain::chunk_span(map_generator);
    let half = map_generator.map_chunk_size as f32 / 2.0;

    // Vertex positions are at n - 0.5 for integer n.
    // Bilinear interpolation between the 4 surrounding vertices.
    let n_x = (world_pos.x + 0.5).floor() as i64;
    let n_z = (world_pos.z + 0.5).floor() as i64;

    let frac_x = world_pos.x - (n_x as f32 - 0.5);
    let frac_z = world_pos.z - (n_z as f32 - 0.5);

    let h00 = vertex_total_height(n_x, n_z, terrain_sampler, map_generator, sculpt_map, chunk_span_val, half);
    let h10 = vertex_total_height(n_x + 1, n_z, terrain_sampler, map_generator, sculpt_map, chunk_span_val, half);
    let h01 = vertex_total_height(n_x, n_z + 1, terrain_sampler, map_generator, sculpt_map, chunk_span_val, half);
    let h11 = vertex_total_height(n_x + 1, n_z + 1, terrain_sampler, map_generator, sculpt_map, chunk_span_val, half);

    let h0 = h00 + (h10 - h00) * frac_x;
    let h1 = h01 + (h11 - h01) * frac_x;
    h0 + (h1 - h0) * frac_z
}

pub(crate) fn sync_grid_heights(
    world_grid: &mut WorldGrid,
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
    sculpt_map: &SculptMap,
) {
    for (_pos, tile) in world_grid.tiles.iter_mut() {
        let world = _pos.grid_to_world();
        tile.terrain_height =
            sample_total_height(terrain_sampler, map_generator, sculpt_map, world);
    }
}

fn draw_custom_cursor(
    brush_config: Res<BrushConfig>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    terrain_sampler: Res<TerrainSampler>,
    map_generator: Res<Persistent<MapGenerator>>,
    sculpt_map: Res<SculptMap>,
    mut gizmos: Gizmos,
) {
    if !brush_config.active {
        return;
    }

    let (camera, camera_transform) = camera_query.into_inner();
    let Some(hit_pos) = get_terrain_hit_position_with_sculpt(
        camera,
        camera_transform,
        &window,
        &terrain_sampler,
        &map_generator,
        &sculpt_map,
    ) else {
        return;
    };

    let color = match brush_config.tool {
        TerrainTool::Raise => Color::srgb(0.2, 0.6, 1.0),
        TerrainTool::Lower => Color::srgb(1.0, 0.3, 0.2),
        TerrainTool::Flatten => Color::srgb(0.3, 1.0, 0.3),
        TerrainTool::Smooth => Color::srgb(1.0, 0.8, 0.2),
    };

    let segments = 64;
    let lift = 0.1;
    let mut prev: Option<Vec3> = None;
    for i in 0..=segments {
        let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
        let x = hit_pos.x + brush_config.radius * angle.cos();
        let z = hit_pos.z + brush_config.radius * angle.sin();
        let h = sample_total_height(
            &terrain_sampler,
            &map_generator,
            &sculpt_map,
            Vec3::new(x, 0.0, z),
        );
        let p = Vec3::new(x, h + lift, z);
        if let Some(prev) = prev {
            gizmos.line(prev, p, color);
        }
        prev = Some(p);
    }
}