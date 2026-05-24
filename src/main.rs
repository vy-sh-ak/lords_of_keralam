use bevy::prelude::*;

mod camera_plugin;
mod compass;
mod terrain;
mod world_direction;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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
        Transform::from_xyz(0.5, 12.0, 0.5).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));

    let chunk = terrain::generate_chunk();

    let tile_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("tile.glb"));

    terrain::spawn_chunk_tiles(&mut commands, &chunk, tile_scene);

    let hut_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("hut.glb"));
    terrain::spawn_hut_at_tile(
        &mut commands,
        &chunk,
        terrain::TileCoord::new(3, 3),
        hut_scene,
    );
    commands.insert_resource(chunk);
    // light
    commands.spawn((
        Name::new("Light"),
        PointLight {
            shadows_enabled: true,
            ..PointLight::default()
        },
        Transform::from_xyz(3.0, 8.0, 5.0),
    ));

    compass::spawn_compass(commands, asset_server);
}
