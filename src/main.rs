use bevy::{light::CascadeShadowConfigBuilder, prelude::*};
use bevy_voxel_world::prelude::*;

mod camera_plugin;
mod compass;
mod terrain;
mod world_direction;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VoxelWorldPlugin::with_config(terrain::MainWorld))
        .add_plugins(camera_plugin::CameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            compass::update_compass_system.after(camera_plugin::CameraSystems::UpdateState),
        )
        .run();
}
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Projection::from(PerspectiveProjection::default()),
        Transform::from_xyz(terrain::CAMERA_VEC3.x, terrain::CAMERA_VEC3.y, terrain::CAMERA_VEC3.z).looking_at(Vec3::ZERO, Vec3::Y),
        // This tells bevy_voxel_world to use this cameras transform to calculate spawning area
        VoxelWorldCamera::<terrain::MainWorld>::default(),
    ));

    // Sun
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,
        maximum_distance: 3.0,
        ..default()
    }
    .build();
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
        cascade_shadow_config,
    ));

    compass::spawn_compass(commands, asset_server);
}
