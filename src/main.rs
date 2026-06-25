use bevy::{
    asset::AssetPlugin,
    diagnostic::FrameTimeDiagnosticsPlugin,
    light::CascadeShadowConfigBuilder,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        RenderPlugin,
        render_resource::WgpuFeatures,
        settings::{RenderCreation, WgpuSettings},
    },
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_persistent::Persistent;
use camera_config::CameraSettings;

mod camera_config;
mod compass;
mod map_asset;
mod persistence;
mod terrain;
mod terrain_painter;
mod ui_editor;
mod world_direction;
mod editor_config;
mod world_grid_config;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                // .set(AssetPlugin {
                //     watch_for_changes_override: Some(true),
                //     ..default()
                // })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                }),
            FrameTimeDiagnosticsPlugin::default(),
            EguiPlugin::default(),
            WireframePlugin::default(),
            DefaultInspectorConfigPlugin,
        ))
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::BLACK.into(),
        })
        .add_plugins(camera_config::CameraPlugin)
        .add_plugins(terrain::MapGenerator::default().plugin())
        .add_plugins(terrain::EndlessTerrainPlugin)
        .add_plugins(terrain_painter::TerrainPainterPlugin)
        .insert_resource(map_asset::ActiveMap::default())
        .add_plugins(world_grid_config::GridGeneratorPlugin)
        .add_plugins(
            ui_editor::UIEditor::default()
                .with_toggle_key(KeyCode::F1)
                .starts_open(true)
                .plugin(),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (compass::update_compass_system.after(camera_config::CameraSystems::UpdateState),),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_settings: Res<Persistent<CameraSettings>>,
) {
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Projection::from(PerspectiveProjection::default()),
        Transform::from_translation(camera_settings.start_position)
            .looking_at(camera_settings.start_look_at, Vec3::Y),
    ));
    // debugging cube
    commands.spawn((
        Name::new("Cube"),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 20.0, 0.0),
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

    compass::spawn_compass(commands, asset_server);
}
