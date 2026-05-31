use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin, light::CascadeShadowConfigBuilder, prelude::*,
    text::DEFAULT_FONT_DATA,
};
use bevy_voxel_world::{custom_meshing::CHUNK_SIZE_F, prelude::*};

mod camera_plugin;
mod compass;
mod terrain;
mod world_direction;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin::default()))
        .add_plugins(VoxelWorldPlugin::with_config(terrain::MainWorld::default()))
        .add_plugins(camera_plugin::CameraPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                compass::update_compass_system.after(camera_plugin::CameraSystems::UpdateState),
                terrain::update_fps_text,
            ),
        )
        .run();
}
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut fonts: ResMut<Assets<Font>>) {
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Projection::from(PerspectiveProjection::default()),
        Transform::from_xyz(
            terrain::CAMERA_VEC3.x,
            terrain::CAMERA_VEC3.y,
            terrain::CAMERA_VEC3.z,
        )
        .looking_at(Vec3::ZERO, Vec3::Y),
        // This tells bevy_voxel_world to use this cameras transform to calculate spawning area
        VoxelWorldCamera::<terrain::MainWorld>::default(),
        DistanceFog {
            color: *ClearColor::default(),
            falloff: FogFalloff::Linear {
                start: 125.0 * CHUNK_SIZE_F,
                end: 200.0 * CHUNK_SIZE_F,
            },
            ..default()
        },
    ));

    // Sun
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 200.0,
        maximum_distance: 3000.0,
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

    // Ambient light, same color as sun
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.98, 0.95, 0.82),
        brightness: 100.0,
        affects_lightmapped_meshes: true,
    });

    // UI overlay camera
    // commands.spawn((
    //     Camera2d,
    //     Camera {
    //         order: 1,
    //         ..default()
    //     },
    // ));

    let font = fonts.add(Font::try_from_bytes(DEFAULT_FONT_DATA.to_vec()).unwrap());
    commands.spawn((
        Text::new("FPS: -- (--)\nFrame: -- ms"),
        TextFont {
            font,
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(12.0),
            ..default()
        },
        terrain::FpsText,
    ));

    compass::spawn_compass(commands, asset_server);
}
