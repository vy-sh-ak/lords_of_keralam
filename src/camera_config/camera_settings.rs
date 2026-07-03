use std::f32::consts::PI;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Resource, Clone, Serialize, Deserialize, PartialEq, Reflect)]
pub struct CameraSettings {
    pub zoom: f32,
    pub target_zoom: f32,
    pub zoom_speed: f32,
    pub zoom_smoothness: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_elevation: f32,
    pub max_elevation: f32,
    pub focus_xz: Vec2,
    pub focus_height: f32,
    pub move_speed_zoomed_in: f32,
    pub move_speed_zoomed_out: f32,
    pub orbit_yaw: f32,
    pub orbit_pitch: f32,
    pub target_orbit_pitch: f32,
    pub orbit_rotate_sensitivity: f32,
    pub vertical_rotate_sensitivity: f32,
    pub orbit_pitch_smoothness: f32,
    pub rts_enabled: bool,
    pub start_position: Vec3,
    pub start_look_at: Vec3,
}

pub const DEFAULT_ORBIT_PITCH: f32 = 0.0;

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            target_zoom: 0.0,
            zoom_speed: 0.05,
            zoom_smoothness: 12.0,
            min_distance: 6.0,
            max_distance: 500.0,
            min_elevation: 0.0,
            max_elevation: PI / 2.0 - 0.05,
            focus_xz: Vec2::new(3.0, 3.0),
            focus_height: 6.0,
            move_speed_zoomed_in: 50.0,
            move_speed_zoomed_out: 200.0,
            orbit_yaw: PI / 4.0,
            orbit_pitch: DEFAULT_ORBIT_PITCH,
            target_orbit_pitch: DEFAULT_ORBIT_PITCH,
            orbit_rotate_sensitivity: 0.01,
            vertical_rotate_sensitivity: 0.02,
            orbit_pitch_smoothness: 12.0,
            rts_enabled: true,
            start_position: Vec3::new(0.0, 2.0, 0.0),
            start_look_at: Vec3::ZERO,
        }
    }
}
