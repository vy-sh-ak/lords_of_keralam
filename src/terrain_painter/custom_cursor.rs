

use bevy::prelude::*;

use bevy_persistent::Persistent;

use crate::terrain::{MapGenerator, TerrainSampler};
use super::sculpt_map::{get_terrain_hit_position_with_sculpt, SculptMap, sample_total_height};
use super::brush::{BrushConfig, TerrainTool};

pub fn draw_custom_cursor(
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