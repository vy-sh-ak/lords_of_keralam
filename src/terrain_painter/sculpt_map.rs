
use std::collections::HashMap;

use bevy::math::Ray3d;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use bevy_persistent::Persistent;

use crate::terrain::endless_terrain::{EndlessTerrainState};
use crate::terrain::{MapGenerator, TerrainSampler, chunk_span};
use crate::ui_editor::UIKeyboardCapture;

use super::brush::{BrushConfig, TerrainTool, apply_brush};
use super::undo_redo::{UndoEntry, UndoStack};

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


pub fn sculpt_paint_system(
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
    let affected_coords = get_affected_chunks(
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
    let affected = apply_brush(
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

pub fn get_terrain_hit_position_with_sculpt(
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

pub fn get_affected_chunks(
    hit_pos: Vec3,
    radius: f32,
    map_generator: &MapGenerator,
) -> HashSet<IVec2> {
    let chunk_span_val = chunk_span(map_generator);
    let hit_chunk = IVec2::new(
        (hit_pos.x / chunk_span_val).round() as i32,
        (hit_pos.z / chunk_span_val).round() as i32,
    );
    let chunk_radius_span = (radius / chunk_span_val).ceil() as i32 + 1;

    let mut affected = HashSet::new();
    for cy in (hit_chunk.y - chunk_radius_span)..=(hit_chunk.y + chunk_radius_span) {
        for cx in (hit_chunk.x - chunk_radius_span)..=(hit_chunk.x + chunk_radius_span) {
            let coord = IVec2::new(cx, cy);
            let center_x = cx as f32 * chunk_span_val;
            let center_z = cy as f32 * chunk_span_val;
            let half_span = chunk_span_val / 2.0;
            if (hit_pos.x - center_x).abs() > half_span + radius
                || (hit_pos.z - center_z).abs() > half_span + radius
            {
                continue;
            }
            affected.insert(coord);
        }
    }
    affected
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

pub fn ray_intersect_terrain(
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

pub fn sample_total_height(
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
