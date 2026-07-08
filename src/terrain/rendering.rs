use bevy::{
    prelude::*,
};
use bevy_persistent::Persistent;

use crate::terrain::{
    DrawMode, FallOffGenerator, MapGenerator, MeshGenerator, TerrainSampler,
    generate_map_data,
    generation::ensure_falloff_map,
    terrain_material::{TerrainMaterial, build_terrain_material},
    texture_generator::texture_from_height_map,
    wireframe,
};

struct RenderAssets {
    mesh: Mesh,
    texture: Option<Image>,
    vertical_offset: f32,
    show_wireframe: bool,
}

#[derive(Component)]
pub(crate) struct GroundPlane;

pub(crate) fn setup_noise_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    terrain_sampler: Res<TerrainSampler>,
    mut map_generator: ResMut<Persistent<MapGenerator>>,
) {
    ensure_falloff_map(&mut map_generator);
    if map_generator.draw_mode == DrawMode::EndlessTerrain {
        return;
    }
    let render_assets = create_render_assets(&map_generator, terrain_sampler.as_ref());
    spawn_ground_plane(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        &mut images,
        &asset_server,
        &mut terrain_materials,
        &render_assets,
        &map_generator,
    );
}

pub(crate) fn refresh_noise_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    ground_plane_entities: Query<Entity, With<GroundPlane>>,
    map_generator: Res<Persistent<MapGenerator>>,
    mut ground_plane_query: Query<(Entity, &mut Mesh3d, &mut Transform), With<GroundPlane>>,
    terrain_sampler: Res<TerrainSampler>,
) {
    if map_generator.draw_mode == DrawMode::EndlessTerrain {
        for entity in &ground_plane_entities {
            commands.entity(entity).despawn();
        }
        return;
    }

    let render_assets = create_render_assets(&map_generator, terrain_sampler.as_ref());

    let entity = match ground_plane_query.single_mut() {
        Ok((entity, mut mesh_handle, mut transform)) => {
            *mesh_handle = Mesh3d(meshes.add(render_assets.mesh.clone()));
            transform.translation = Vec3::new(0.0, render_assets.vertical_offset, 0.0);
            entity
        }
        Err(_) => {
            spawn_ground_plane(
                &mut commands,
                &mut meshes,
                &mut standard_materials,
                &mut images,
                &asset_server,
                &mut terrain_materials,
                &render_assets,
                &map_generator,
            );
            return;
        }
    };

    match map_generator.draw_mode {
        DrawMode::Mesh => {
            commands
                .entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>();
            let mat_handle =
                build_terrain_material(&mut terrain_materials, &asset_server, &map_generator);
            commands.entity(entity).insert(MeshMaterial3d(mat_handle));
        }
        _ => {
            commands
                .entity(entity)
                .remove::<MeshMaterial3d<TerrainMaterial>>();
            if let Some(texture) = &render_assets.texture {
                let texture_handle = images.add(texture.clone());
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(standard_materials.add(StandardMaterial {
                        base_color_texture: Some(texture_handle),
                        perceptual_roughness: 1.0,
                        ..default()
                    })));
            }
        }
    }

    wireframe::apply_wireframe_debug(&mut commands.entity(entity), render_assets.show_wireframe);
}

fn spawn_ground_plane(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    standard_materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    asset_server: &AssetServer,
    terrain_materials: &mut Assets<TerrainMaterial>,
    render_assets: &RenderAssets,
    map_generator: &MapGenerator,
) {
    let mesh_handle = meshes.add(render_assets.mesh.clone());
    let mut entity_commands = commands.spawn((
        GroundPlane,
        Name::new("GroundPlane"),
        Mesh3d(mesh_handle),
        Transform::from_xyz(0.0, render_assets.vertical_offset, 0.0),
    ));

    match map_generator.draw_mode {
        DrawMode::Mesh => {
            let mat_handle = build_terrain_material(terrain_materials, asset_server, map_generator);
            info!(
                "Spawned GroundPlane with TerrainMaterial handle={:?}",
                mat_handle
            );
            entity_commands.insert(MeshMaterial3d(mat_handle));
        }
        DrawMode::NoiseMap | DrawMode::FallOffMap => {
            if let Some(texture) = &render_assets.texture {
                let texture_handle = images.add(texture.clone());
                entity_commands.insert(MeshMaterial3d(standard_materials.add(StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    perceptual_roughness: 1.0,
                    ..default()
                })));
            }
        }
        _ => {}
    }

    wireframe::apply_wireframe_debug(&mut entity_commands, render_assets.show_wireframe);
}

fn create_render_assets(
    map_generator: &MapGenerator,
    terrain_sampler: &TerrainSampler,
) -> RenderAssets {
    let map_data = generate_map_data(terrain_sampler, IVec2::ZERO, map_generator);
    info!(
        "Generated noise map for render assets {draw_mode:?}",
        draw_mode = map_generator.draw_mode
    );
    info!("Noise map: {:?}", map_data.noise_map.iter().map(|row| row.iter().take(5).cloned().collect::<Vec<f32>>()).take(5).collect::<Vec<Vec<f32>>>());
    match map_generator.draw_mode {
        DrawMode::NoiseMap => RenderAssets {
            mesh: create_plane_mesh(map_generator),
            texture: Some(texture_from_height_map(&map_data.noise_map)),
            vertical_offset: -0.01,
            show_wireframe: map_generator.show_uv_wireframe,
        },
        DrawMode::Mesh => RenderAssets {
            mesh: MeshGenerator::generate_terrain_mesh(
                &map_data.noise_map,
                map_generator.terrain_data.height_multiplier,
                &map_generator.terrain_data.height_curve,
                map_generator.level_of_detail,
                None,
            )
            .create_mesh(),
            texture: None,
            vertical_offset: 0.0,
            show_wireframe: map_generator.show_uv_wireframe,
        },
        DrawMode::EndlessTerrain => RenderAssets {
            mesh: create_plane_mesh(map_generator),
            texture: Some(texture_from_height_map(&map_data.noise_map)),
            vertical_offset: -0.01,
            show_wireframe: map_generator.show_uv_wireframe,
        },
        DrawMode::FallOffMap => {
            let fall_off_map =
                FallOffGenerator::generate_fall_off_map(map_generator.map_chunk_size as usize);
            RenderAssets {
                mesh: create_plane_mesh(map_generator),
                texture: Some(texture_from_height_map(&fall_off_map)),
                vertical_offset: -0.01,
                show_wireframe: map_generator.show_uv_wireframe,
            }
        }
    }
}

fn create_plane_mesh(map_generator: &MapGenerator) -> Mesh {
    Plane3d::default()
        .mesh()
        .size(
            map_generator.map_chunk_size as f32,
            map_generator.map_chunk_size as f32,
        )
        .into()
}