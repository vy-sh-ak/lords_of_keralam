use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::terrain::{MapGenerator, TerrainSampler};
use crate::terrain_painter::{ray_intersect_terrain, sample_total_height, SculptMap};
use crate::ui_editor::UIKeyboardCapture;
use crate::world_grid_config::{TilePos, TILE_SIZE};

use super::build_mode::BuildMode;

pub fn highlight_hovered_tile(
    build_mode: Res<BuildMode>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    terrain_sampler: Res<TerrainSampler>,
    map_generator: Res<Persistent<MapGenerator>>,
    sculpt_map: Res<SculptMap>,
    mut gizmos: Gizmos,
    keyboard_capture: Option<Res<UIKeyboardCapture>>,
) {
    if !build_mode.enabled {
        return;
    }

    if let Some(capture) = keyboard_capture.as_ref() {
        if capture.wants_pointer_input || capture.is_typing {
            return;
        }
    }

    let (camera, camera_transform) = camera_query.into_inner();
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        return;
    };
    let Some(hit_pos) = ray_intersect_terrain(&ray, &terrain_sampler, &map_generator, Some(&sculpt_map)) else {
        return;
    };

    let tile_pos = TilePos::world_to_grid(hit_pos);
    let min = tile_pos.grid_to_world();

    let lift = 0.1;
    let color = Color::srgb(1.0, 0.85, 0.0);

    let raw_corners = [
        Vec3::new(min.x, 0.0, min.z),
        Vec3::new(min.x + TILE_SIZE, 0.0, min.z),
        Vec3::new(min.x + TILE_SIZE, 0.0, min.z + TILE_SIZE),
        Vec3::new(min.x, 0.0, min.z + TILE_SIZE),
    ];

    let corners = raw_corners.map(|p| {
        let h = sample_total_height(&terrain_sampler, &map_generator, &sculpt_map, p);
        Vec3::new(p.x, h + lift, p.z)
    });

    for i in 0..4 {
        gizmos.line(corners[i], corners[(i + 1) % 4], color);
    }
}
