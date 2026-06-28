pub mod brush;

use std::collections::HashMap;

use bevy::math::Ray3d;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_persistent::Persistent;

use crate::terrain::endless_terrain::{sync_endless_terrain, EndlessTerrainState};
use crate::terrain::{MapGenerator, TerrainSampler};
use crate::ui_editor::UIKeyboardCapture;
use crate::world_grid_config::WorldGrid;

mod sculpt_map_serde {
    use std::collections::HashMap;

    use bevy::math::IVec2;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(
        chunks: &HashMap<IVec2, Vec<Vec<f32>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let pairs: Vec<(String, Vec<Vec<f32>>)> = chunks
            .iter()
            .map(|(coord, data)| (format!("{},{}", coord.x, coord.y), data.clone()))
            .collect();
        pairs.serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<IVec2, Vec<Vec<f32>>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let pairs: Vec<(String, Vec<Vec<f32>>)> = Vec::deserialize(deserializer)?;
        let mut map = HashMap::with_capacity(pairs.len());
        for (key, data) in pairs {
            if let Some((x, y)) = key.split_once(',') {
                if let (Ok(x), Ok(y)) = (x.parse::<i32>(), y.parse::<i32>()) {
                    map.insert(IVec2::new(x, y), data);
                }
            }
        }
        Ok(map)
    }
}

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct SculptMap {
    #[serde(with = "sculpt_map_serde")]
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

#[derive(Clone)]
pub struct UndoEntry {
    pub old_chunks: HashMap<IVec2, Option<Vec<Vec<f32>>>>,
}

#[derive(Resource)]
pub struct UndoStack {
    pub undo_entries: Vec<UndoEntry>,
    pub redo_entries: Vec<UndoEntry>,
    max_entries: usize,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self {
            undo_entries: Vec::new(),
            redo_entries: Vec::new(),
            max_entries: 100,
        }
    }
}

impl UndoStack {
    pub fn push(&mut self, entry: UndoEntry) {
        self.redo_entries.clear();
        self.undo_entries.push(entry);
        if self.undo_entries.len() > self.max_entries {
            self.undo_entries.remove(0);
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_entries.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_entries.is_empty()
    }
}

pub struct TerrainPainterPlugin;

impl Plugin for TerrainPainterPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SculptMap::default())
            .insert_resource(BrushConfig::default())
            .insert_resource(SyncGridRequest::default())
            .insert_resource(UndoStack::default())
            .add_systems(Update, sculpt_paint_system.before(sync_endless_terrain))
            .add_systems(Update, draw_custom_cursor)
            .add_systems(Update, sync_grid_system)
            .add_systems(
                Update,
                undo_redo_system.before(sculpt_paint_system),
            );
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
    mut undo_stack: ResMut<UndoStack>,
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

    // Snapshot affected chunks before applying brush (for undo)
    let affected_coords = brush::get_affected_chunks(
        hit_pos,
        brush_config.radius,
        &map_generator,
    );
    let mut snapshot = HashMap::new();
    for coord in &affected_coords {
        let old = sculpt_map.chunks.get(coord).cloned();
        snapshot.insert(*coord, old);
    }

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

    if !affected.is_empty() {
        undo_stack.push(UndoEntry {
            old_chunks: snapshot,
        });
    }

    for coord in affected {
        endless_state.mark_chunk_dirty(coord);
    }
}

fn undo_redo_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut sculpt_map: ResMut<SculptMap>,
    mut undo_stack: ResMut<UndoStack>,
    mut endless_state: ResMut<EndlessTerrainState>,
    mut sync_request: ResMut<SyncGridRequest>,
    brush_config: Res<BrushConfig>,
    keyboard_capture: Option<Res<UIKeyboardCapture>>,
) {
    if !brush_config.active {
        return;
    }
    if let Some(capture) = keyboard_capture.as_ref() {
        if capture.is_typing {
            return;
        }
    }

    let ctrl = keyboard.pressed(KeyCode::ControlLeft)
        || keyboard.pressed(KeyCode::ControlRight);
    let shift = keyboard.pressed(KeyCode::ShiftLeft)
        || keyboard.pressed(KeyCode::ShiftRight);

    if ctrl && !shift && keyboard.just_pressed(KeyCode::KeyZ) {
        // Undo
        let entry = match undo_stack.undo_entries.pop() {
            Some(e) => e,
            None => return,
        };
        // Save current state of affected chunks for redo
        let mut redo_chunks = HashMap::new();
        for &coord in entry.old_chunks.keys() {
            redo_chunks.insert(coord, sculpt_map.chunks.get(&coord).cloned());
        }
        undo_stack.redo_entries.push(UndoEntry {
            old_chunks: redo_chunks,
        });
        // Restore old state
        for (coord, old_data) in entry.old_chunks {
            match old_data {
                Some(data) => {
                    sculpt_map.chunks.insert(coord, data);
                }
                None => {
                    sculpt_map.chunks.remove(&coord);
                }
            }
            endless_state.mark_chunk_dirty(coord);
        }
        sync_request.0 = true;
    } else if ctrl && shift && keyboard.just_pressed(KeyCode::KeyZ) {
        // Redo
        let entry = match undo_stack.redo_entries.pop() {
            Some(e) => e,
            None => return,
        };
        let mut undo_chunks = HashMap::new();
        for &coord in entry.old_chunks.keys() {
            undo_chunks.insert(coord, sculpt_map.chunks.get(&coord).cloned());
        }
        undo_stack.undo_entries.push(UndoEntry {
            old_chunks: undo_chunks,
        });
        for (coord, old_data) in entry.old_chunks {
            match old_data {
                Some(data) => {
                    sculpt_map.chunks.insert(coord, data);
                }
                None => {
                    sculpt_map.chunks.remove(&coord);
                }
            }
            endless_state.mark_chunk_dirty(coord);
        }
        sync_request.0 = true;
    }
}

fn sample_terrain(
    pos: Vec3,
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
    sculpt_map: Option<&SculptMap>,
) -> f32 {
    let sample_pos = Vec3::new(pos.x, 0.0, pos.z);
    match sculpt_map {
        Some(sculpt) => sample_total_height(terrain_sampler, map_generator, sculpt, sample_pos),
        None => terrain_sampler.sample_height(map_generator, sample_pos.x, sample_pos.z),
    }
}

pub(crate) fn ray_intersect_terrain(
    ray: &Ray3d,
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
    sculpt_map: Option<&SculptMap>,
) -> Option<Vec3> {
    let max_dist = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Dir3::Y))?;
    let mut near = 0.0;
    let mut far = max_dist;

    for _ in 0..20 {
        let t = (near + far) / 2.0;
        let point = ray.get_point(t);
        let terrain_height = sample_terrain(point, terrain_sampler, map_generator, sculpt_map);
        let diff = point.y - terrain_height;
        if diff.abs() < 0.01 {
            return Some(Vec3::new(point.x, terrain_height, point.z));
        }
        if diff > 0.0 {
            near = t;
        } else {
            far = t;
        }
    }

    let point = ray.get_point((near + far) / 2.0);
    let terrain_height = sample_terrain(point, terrain_sampler, map_generator, sculpt_map);
    Some(Vec3::new(point.x, terrain_height, point.z))
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
    ray_intersect_terrain(&ray, terrain_sampler, map_generator, None)
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
    ray_intersect_terrain(&ray, terrain_sampler, map_generator, Some(sculpt_map))
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