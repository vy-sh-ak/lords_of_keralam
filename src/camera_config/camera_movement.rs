use std::f32::consts::PI;

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};
use bevy_persistent::Persistent;

use crate::{
    ui_editor::UIKeyboardCapture,
};

use super::CameraSettings;
use super::DEFAULT_ORBIT_PITCH;

pub fn move_focus(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    keyboard_capture: Option<Res<UIKeyboardCapture>>,
    mut camera_settings: ResMut<Persistent<CameraSettings>>,
) {
    if keyboard_capture
        .as_ref()
        .is_some_and(|capture| capture.is_typing)
    {
        return;
    }

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

    let delta = movement * movement_speed * time.delta_secs();

    camera_settings.focus_xz += delta.xz();
}

pub fn zoom(
    time: Res<Time>,
    camera: Single<&mut Transform, With<Camera>>,
    mut camera_settings: ResMut<Persistent<CameraSettings>>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
    keyboard_capture: Option<Res<UIKeyboardCapture>>,
) {
    let delta = if keyboard_capture
        .as_ref()
        .is_some_and(|capture| capture.wants_pointer_input)
    {
        0.0
    } else {
        mouse_wheel_input.delta.y * camera_settings.zoom_speed
    };

    camera_settings.target_zoom = (camera_settings.target_zoom + delta).clamp(0.0, 1.0);

    let alpha = 1.0 - (-camera_settings.zoom_smoothness * time.delta_secs()).exp();
    camera_settings.zoom += (camera_settings.target_zoom - camera_settings.zoom) * alpha;

    if (camera_settings.target_zoom - camera_settings.zoom).abs() < 0.001 {
        camera_settings.zoom = camera_settings.target_zoom;
    }

    let t = camera_settings.zoom;

    let pitch = if camera_settings.rts_enabled {
        if camera_settings.target_zoom == 0.0 {
            let pitch_reset_alpha = 1.0 - (-8.0 * time.delta_secs()).exp();
            camera_settings.target_orbit_pitch +=
                (DEFAULT_ORBIT_PITCH - camera_settings.target_orbit_pitch) * pitch_reset_alpha;

            if (camera_settings.target_orbit_pitch - DEFAULT_ORBIT_PITCH).abs() < 0.001 {
                camera_settings.target_orbit_pitch = DEFAULT_ORBIT_PITCH;
            }
        }

        let alpha = 1.0 - (-camera_settings.orbit_pitch_smoothness * time.delta_secs()).exp();
        camera_settings.orbit_pitch += (camera_settings.target_orbit_pitch - camera_settings.orbit_pitch) * alpha;

        let base_pitch = camera_settings.max_elevation
            + (camera_settings.min_elevation - camera_settings.max_elevation) * t;

        (base_pitch + camera_settings.orbit_pitch).clamp(-0.2, camera_settings.max_elevation)
    } else {
        let alpha = 1.0 - (-camera_settings.orbit_pitch_smoothness * time.delta_secs()).exp();
        camera_settings.orbit_pitch += (camera_settings.target_orbit_pitch - camera_settings.orbit_pitch) * alpha;

        (camera_settings.max_elevation - camera_settings.orbit_pitch)
            .clamp(0.1, camera_settings.max_elevation)
    };

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

    let target = Vec3::new(
        camera_settings.focus_xz.x,
        camera_settings.focus_height,
        camera_settings.focus_xz.y,
    );

    let mut transform = camera.into_inner();
    let new_translation = target + yaw_direction * horizontal_dist + Vec3::Y * height;
    let translation_changed = transform.translation.distance_squared(new_translation) > 0.0001;

    if translation_changed {
        transform.translation = new_translation;
        transform.look_at(target, Vec3::Y);
    }
}

pub fn rotate_horizontal(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    keyboard_capture: Option<Res<UIKeyboardCapture>>,
    mut camera_settings: ResMut<Persistent<CameraSettings>>,
) {
    if mouse_buttons.pressed(MouseButton::Middle)
        && !keyboard_capture
            .as_ref()
            .is_some_and(|c| c.wants_pointer_input)
    {
        let delta_x = mouse_motion.delta.x;
        if delta_x != 0.0 {
            camera_settings.orbit_yaw += delta_x * camera_settings.orbit_rotate_sensitivity;
        }
    }
}

pub fn rotate_vertical(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    keyboard_capture: Option<Res<UIKeyboardCapture>>,
    mut camera_settings: ResMut<Persistent<CameraSettings>>,
) {
    if mouse_buttons.pressed(MouseButton::Middle)
        && !keyboard_capture
            .as_ref()
            .is_some_and(|c| c.wants_pointer_input)
    {
        let delta_y = mouse_motion.delta.y;
        if delta_y != 0.0
            && (camera_settings.rts_enabled && camera_settings.target_zoom >= 1.0
                || !camera_settings.rts_enabled)
        {
            camera_settings.target_orbit_pitch = (camera_settings.target_orbit_pitch
                - delta_y * camera_settings.vertical_rotate_sensitivity)
                .clamp(-0.5, PI / 2.0);
        }
    }
}
