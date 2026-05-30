use bevy::{light::CascadeShadowConfigBuilder, platform::collections::HashSet, prelude::*};
use noiz::{
    prelude::{common_noise::Perlin, *},
    rng::NoiseRng,
};

mod camera_plugin;
mod compass;
mod terrain;
mod world_direction;

const TILE_W: usize = 4;
const TILE_H: usize = 4;

const GRID_COLS: usize = 100;
const GRID_ROWS: usize = 100;

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
#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut noise = Noise::<PerCell<OrthoGrid, Random<UNorm, f32>>>::default();
    noise.set_seed(42);
    noise.set_frequency(2.0);

    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Projection::from(PerspectiveProjection::default()),
        Transform::from_xyz(0.5, 12.0, 0.5).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    let mut tiles = HashSet::new();
    for x in 0..GRID_COLS {
        for z in 0..GRID_ROWS {
            let random_unorm: f32 = noise.sample(Vec2::new(x as f32, z as f32));
            println!("Noise value at ({}, {}): {}", x, z, random_unorm);
            if random_unorm < 0.2 {
                continue;
            }
            tiles.insert((x, z));
        }
    }

    for (x, z) in tiles {
        commands.spawn((
            Name::new(format!("Tile ({}, {})", x, z)),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(1., 1.))),
            MeshMaterial3d(materials.add(Color::srgb(0.4, 0.6, 0.4))),
            Transform::from_xyz(x as f32, 0.0, z as f32),
        ));
    }

    // let chunk = terrain::generate_chunk();

    // let tile_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("tile.glb"));

    // terrain::spawn_chunk_tiles(&mut commands, &chunk, tile_scene);

    // let hut_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("hut.glb"));
    // terrain::spawn_hut_at_tile(
    //     &mut commands,
    //     &chunk,
    //     terrain::TileCoord::new(3, 3),
    //     hut_scene,
    // );
    // commands.insert_resource(chunk);
    // sun light
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
