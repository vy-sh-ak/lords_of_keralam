use std::f32::consts::PI;

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};

use crate::world_direction::WorldDirection;

const FOCUS_DEFAULTS: Vec3 = Vec3::new(3.0, 6.0, 3.0);
const DEFAULT_ORBIT_PITCH: f32 = 0.0;

#[derive(Debug, Resource)]
pub struct CameraSettings {
    pub zoom: f32,
    pub target_zoom: f32,
    pub zoom_speed: f32,
    pub zoom_smoothness: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_elevation: f32,
    pub max_elevation: f32,
    pub focus: Vec3,
    pub move_speed_zoomed_in: f32,
    pub move_speed_zoomed_out: f32,
    pub orbit_yaw: f32,
    pub orbit_pitch: f32,
    pub orbit_rotate_sensitivity: f32,
    pub vertical_rotate_sensitivity: f32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CameraSystems {
    UpdateState,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        let camera_settings = CameraSettings {
            zoom: 1.0,
            target_zoom: 1.0,
            zoom_speed: 0.05,
            zoom_smoothness: 12.0,
            min_distance: 6.0,
            max_distance: 500.0,
            min_elevation: 0.0,
            max_elevation: PI / 2.0 - 0.05,
            focus: FOCUS_DEFAULTS,
            move_speed_zoomed_in: 10.0,
            move_speed_zoomed_out: 200.0,
            orbit_yaw: PI / 4.0,
            orbit_pitch: DEFAULT_ORBIT_PITCH,
            orbit_rotate_sensitivity: 0.01,
            vertical_rotate_sensitivity: 0.02,
        };
        let world_direction = WorldDirection::from_orbit_yaw(camera_settings.orbit_yaw);

        app.insert_resource(camera_settings)
            .insert_resource(world_direction)
            .add_systems(
                Update,
                (
                    (rotate_horizontal, rotate_vertical).chain(),
                    move_focus,
                    zoom,
                    sync_world_direction,
                )
                    .chain()
                    .in_set(CameraSystems::UpdateState),
            );
    }
}

fn move_focus(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_settings: ResMut<CameraSettings>,
) {
    let mut input = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        input.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        input.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        input.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        input.x += 1.0;
    }

    if input == Vec2::ZERO {
        return;
    }

    let orbit_direction = Vec3::new(
        camera_settings.orbit_yaw.cos(),
        0.0,
        camera_settings.orbit_yaw.sin(),
    )
    .normalize();

    let forward = -orbit_direction;
    let right = Vec3::new(-forward.z, 0.0, forward.x).normalize();

    let movement = (forward * input.y + right * input.x).normalize_or_zero();
    let speed_multiplier =
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            3.0
        } else {
            1.0
        };
    let t = camera_settings.zoom.clamp(0.0, 1.0);
    let movement_speed = (camera_settings.move_speed_zoomed_out
        + (camera_settings.move_speed_zoomed_in - camera_settings.move_speed_zoomed_out) * t)
        * speed_multiplier;

    camera_settings.focus += movement * movement_speed * time.delta_secs();
    camera_settings.focus.y = FOCUS_DEFAULTS.y;
}

fn zoom(
    time: Res<Time>,
    camera: Single<&mut Transform, With<Camera>>,
    mut camera_settings: ResMut<CameraSettings>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    let delta = mouse_wheel_input.delta.y * camera_settings.zoom_speed;
    camera_settings.target_zoom = (camera_settings.target_zoom + delta).clamp(0.0, 1.0);

    let alpha = 1.0 - (-camera_settings.zoom_smoothness * time.delta_secs()).exp();
    camera_settings.zoom += (camera_settings.target_zoom - camera_settings.zoom) * alpha;

    if (camera_settings.target_zoom - camera_settings.zoom).abs() < 0.001 {
        camera_settings.zoom = camera_settings.target_zoom;
    }
    if camera_settings.target_zoom == 0.0 {
        let pitch_reset_alpha = 1.0 - (-8.0 * time.delta_secs()).exp();
        camera_settings.orbit_pitch +=
            (DEFAULT_ORBIT_PITCH - camera_settings.orbit_pitch) * pitch_reset_alpha;

        if (camera_settings.orbit_pitch - DEFAULT_ORBIT_PITCH).abs() < 0.001 {
            camera_settings.orbit_pitch = DEFAULT_ORBIT_PITCH;
        }
    }

    let t = camera_settings.zoom;
    let base_pitch = camera_settings.max_elevation
        + (camera_settings.min_elevation - camera_settings.max_elevation) * t;

    let pitch =
        (base_pitch + camera_settings.orbit_pitch).clamp(-0.2, camera_settings.max_elevation);

    let distance = camera_settings.max_distance
        + (camera_settings.min_distance - camera_settings.max_distance) * t;

    let horizontal_dist = distance * pitch.cos();
    let height = distance * pitch.sin();

    let yaw_direction = Vec3::new(
        camera_settings.orbit_yaw.cos(),
        0.0,
        camera_settings.orbit_yaw.sin(),
    )
    .normalize();

    let target = camera_settings.focus;

    let mut transform = camera.into_inner();
    let new_translation = target + yaw_direction * horizontal_dist + Vec3::Y * height;
    let translation_changed = transform.translation.distance_squared(new_translation) > 0.0001;

    if translation_changed {
        transform.translation = new_translation;
        transform.look_at(target, Vec3::Y);
    }
}

fn rotate_horizontal(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera_settings: ResMut<CameraSettings>,
) {
    if !mouse_buttons.pressed(MouseButton::Middle) {
        return;
    }
    let delta_x = mouse_motion.delta.x;
    if delta_x == 0.0 {
        return;
    }
    camera_settings.orbit_yaw += delta_x * camera_settings.orbit_rotate_sensitivity;
}

fn rotate_vertical(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera_settings: ResMut<CameraSettings>,
) {
    if !mouse_buttons.pressed(MouseButton::Middle) {
        return;
    }
    let delta_y = mouse_motion.delta.y;
    if delta_y == 0.0 || camera_settings.target_zoom < 1.0 {
        return;
    }

    camera_settings.orbit_pitch = (camera_settings.orbit_pitch
        + delta_y * camera_settings.vertical_rotate_sensitivity)
        .clamp(-0.1, 1.0);
}

fn sync_world_direction(
    camera_settings: Res<CameraSettings>,
    mut world_direction: ResMut<WorldDirection>,
) {
    world_direction.set_heading_from_orbit_yaw(camera_settings.orbit_yaw);
}
