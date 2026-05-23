use std::{f32::consts::PI, ops::Range};

use bevy::{
    camera::ScalingMode, input::mouse::AccumulatedMouseScroll, math::VectorSpace, prelude::*,
};

#[derive(Debug, Resource)]
struct CameraSettings {
    pub zoom: f32,
    pub zoom_speed: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_elevation: f32,
    pub max_elevation: f32,
    pub focus: Vec3,
    pub move_speed_zoomed_in: f32,
    pub move_speed_zoomed_out: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(CameraSettings {
            zoom: 1.0, // start zoomed in
            zoom_speed: 0.05,
            min_distance: 3.0,
            max_distance: 12.0,
            min_elevation: PI / 12.0,
            max_elevation: PI / 2.0 - 0.05,
            focus: Vec3::new(0.0, 0.5, 0.0),
            move_speed_zoomed_in: 3.0,
            move_speed_zoomed_out: 9.0,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (move_focus, zoom))
        .run();
}
#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Projection::from(PerspectiveProjection::default()),
        Transform::from_xyz(0.5, 12.0, 0.5).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
    commands.spawn((
        Name::new("Plane"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Ground,
    ));

    commands.spawn((
        Name::new("Cube"),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // light
    commands.spawn((
        Name::new("Light"),
        PointLight {
            shadows_enabled: true,
            ..PointLight::default()
        },
        Transform::from_xyz(3.0, 8.0, 5.0),
    ));
}

fn move_focus(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    camera: Single<&Transform, With<Camera>>,
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

    let camera = camera.into_inner();

    let mut forward = camera.forward().as_vec3();
    forward.y = 0.0;
    forward = forward.normalize_or_zero();

    let mut right = camera.right().as_vec3();
    right.y = 0.0;
    right = right.normalize_or_zero();
    let t = camera_settings.zoom.clamp(0.0, 1.0);
    let movement = (forward * input.y + right * input.x).normalize_or_zero();
    let movement_speed = camera_settings.move_speed_zoomed_out
        + (camera_settings.move_speed_zoomed_in - camera_settings.move_speed_zoomed_out) * t;

    camera_settings.focus += movement * movement_speed * time.delta_secs();
    camera_settings.focus.y = 0.5;
}

fn zoom(
    camera: Single<&mut Transform, With<Camera>>,
    mut camera_settings: ResMut<CameraSettings>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    let delta = mouse_wheel_input.delta.y * camera_settings.zoom_speed;
    camera_settings.zoom = (camera_settings.zoom + delta).clamp(0.0, 1.0);
    let t = camera_settings.zoom;
    let elevation = camera_settings.max_elevation
        + (camera_settings.min_elevation - camera_settings.max_elevation) * t;

    let distance = camera_settings.max_distance
        + (camera_settings.min_distance - camera_settings.max_distance) * t;

    let horizontal_dist = distance * elevation.cos();
    let height = distance * elevation.sin();
    // Keep camera on the XZ diagonal (preserves the original camera direction)
    let direction = Vec3::new(1.0, 0.0, 1.0).normalize();
    let target = camera_settings.focus;

    let mut transform = camera.into_inner();
    transform.translation = target + direction * horizontal_dist + Vec3::Y * height;
    transform.look_at(target, Vec3::Y);
}
