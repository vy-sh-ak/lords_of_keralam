pub mod brush;
pub mod ui;

use std::collections::HashMap;

use bevy::prelude::*;

use bevy_persistent::Persistent;

use crate::terrain::endless_terrain::{sync_endless_terrain, EndlessTerrainState};
use crate::terrain::{MapGenerator, TerrainSampler};
use crate::ui_editor::UIKeyboardCapture;

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

pub struct TerrainPainterPlugin;

impl Plugin for TerrainPainterPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SculptMap::default())
            .insert_resource(BrushConfig::default())
            .add_systems(Update, sculpt_paint_system.before(sync_endless_terrain))
            .add_systems(Update, draw_custom_cursor);
    }
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

fn draw_custom_cursor(
    brush_config: Res<BrushConfig>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    terrain_sampler: Res<TerrainSampler>,
    map_generator: Res<Persistent<MapGenerator>>,
    mut gizmos: Gizmos,
) {
    if !brush_config.active {
        return;
    }

    let (camera, camera_transform) = camera_query.into_inner();
    let Some(hit_pos) =
        get_terrain_hit_position(camera, camera_transform, &window, &terrain_sampler, &map_generator)
    else {
        return;
    };

    let color = match brush_config.tool {
        TerrainTool::Raise => Color::srgb(0.2, 0.6, 1.0),
        TerrainTool::Lower => Color::srgb(1.0, 0.3, 0.2),
        TerrainTool::Flatten => Color::srgb(0.3, 1.0, 0.3),
        TerrainTool::Smooth => Color::srgb(1.0, 0.8, 0.2),
    };

    gizmos.circle(
        Isometry3d::new(
            hit_pos + Vec3::Y * 0.05,
            Quat::from_rotation_arc(Vec3::Z, Vec3::Y),
        ),
        brush_config.radius,
        color,
    );
}