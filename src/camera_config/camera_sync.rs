use bevy::{
    prelude::*,
};
use bevy_persistent::Persistent;

use super::world_direction::WorldDirection;
use crate::{
    world_grid_config::{GRID_SIZE, TILE_SIZE},
};
use super::CameraSettings;

pub fn clamp_camera_to_grid(
    mut camera_settings: ResMut<Persistent<CameraSettings>>,
) {
    let max_world = (GRID_SIZE - 1) as f32 * TILE_SIZE;
    camera_settings.focus_xz = camera_settings
        .focus_xz
        .clamp(Vec2::ZERO, Vec2::new(max_world, max_world));
}

pub fn sync_world_direction(
    camera_settings: Res<Persistent<CameraSettings>>,
    mut world_direction: ResMut<WorldDirection>,
) {
    world_direction.set_heading_from_orbit_yaw(camera_settings.orbit_yaw);
}
