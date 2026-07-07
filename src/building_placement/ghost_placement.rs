use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::terrain::{MapGenerator, TerrainSampler};
use crate::terrain_painter::{ray_intersect_terrain, sample_total_height, SculptMap};
use crate::ui_editor::UIKeyboardCapture;
use crate::world_grid_config::{TilePos, WorldGrid, TILE_SIZE};

use super::build_mode::{BuildMode, BuildingSize};
use crate::game_assets::BuildingAssets;

#[derive(Component)]
pub struct GhostBuilding;

#[derive(Component)]
pub struct GhostSize {
    pub tiles_x: u32,
    pub tiles_z: u32,
}

#[derive(Component)]
pub struct GhostMaterials {
    pub gold: Handle<StandardMaterial>,
    pub red: Handle<StandardMaterial>,
}

fn snap_building(pos: Vec3, size: &BuildingSize) -> Vec3 {
    let tile_x = (pos.x / TILE_SIZE).floor();
    let tile_z = (pos.z / TILE_SIZE).floor();
    let w = size.tiles_x as f32 * TILE_SIZE;
    let d = size.tiles_z as f32 * TILE_SIZE;
    let x = tile_x * TILE_SIZE + w / 2.0;
    let z = tile_z * TILE_SIZE + d / 2.0;
    Vec3::new(x, 0.0, z)
}

fn check_overlap(
    world_grid: &WorldGrid,
    center: Vec3,
    size: &BuildingSize,
) -> bool {
    let tile_x = ((center.x - size.tiles_x as f32 * TILE_SIZE / 2.0) / TILE_SIZE).floor() as i32;
    let tile_z = ((center.z - size.tiles_z as f32 * TILE_SIZE / 2.0) / TILE_SIZE).floor() as i32;
    for x in tile_x..tile_x + size.tiles_x as i32 {
        for z in tile_z..tile_z + size.tiles_z as i32 {
            if let Some(tile) = world_grid.tiles.get(&TilePos::new(x, z)) {
                if tile.occupied {
                    return true;
                }
            }
        }
    }
    false
}

fn mark_occupied(
    world_grid: &mut WorldGrid,
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
    sculpt_map: &SculptMap,
    center: Vec3,
    size: &BuildingSize,
) {
    let tile_x = ((center.x - size.tiles_x as f32 * TILE_SIZE / 2.0) / TILE_SIZE).floor() as i32;
    let tile_z = ((center.z - size.tiles_z as f32 * TILE_SIZE / 2.0) / TILE_SIZE).floor() as i32;
    for x in tile_x..tile_x + size.tiles_x as i32 {
        for z in tile_z..tile_z + size.tiles_z as i32 {
            let tile = world_grid.ensure_tile_with_sculpt(
                TilePos::new(x, z),
                terrain_sampler,
                map_generator,
                sculpt_map,
            );
            tile.occupied = true;
        }
    }
}

fn get_terrain_hit(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
    terrain_sampler: &TerrainSampler,
    map_generator: &MapGenerator,
    sculpt_map: &SculptMap,
) -> Option<Vec3> {
    let cursor_pos = window.cursor_position()?;
    let ray = camera.viewport_to_world(camera_transform, cursor_pos).ok()?;
    ray_intersect_terrain(&ray, terrain_sampler, map_generator, Some(sculpt_map))
}

pub fn ghost_placement_system(
    mut build_mode: ResMut<BuildMode>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    terrain_sampler: Res<TerrainSampler>,
    map_generator: Res<Persistent<MapGenerator>>,
    sculpt_map: Res<SculptMap>,
    mut world_grid: ResMut<WorldGrid>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    keyboard_capture: Option<Res<UIKeyboardCapture>>,
    building_assets: Res<BuildingAssets>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ghost_query: Query<
        (
            Entity,
            &mut Transform,
            &mut Visibility,
            &GhostSize,
            &GhostMaterials,
        ),
        With<GhostBuilding>,
    >,
) {
    if build_mode.placing_building.is_none() {
        for (entity, _, _, _, _) in ghost_query.iter_mut() {
            commands.entity(entity).despawn();
        }
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Right) || keyboard.just_pressed(KeyCode::Escape) {
        build_mode.placing_building = None;
        return;
    }

    let ui_active = keyboard_capture
        .as_ref()
        .is_some_and(|k| k.wants_pointer_input || k.is_typing);

    let (camera, camera_transform) = camera_query.into_inner();

    let hit = get_terrain_hit(
        camera,
        camera_transform,
        &window,
        &terrain_sampler,
        &map_generator,
        &sculpt_map,
    );

    let _w = build_mode.building_size.tiles_x as f32 * TILE_SIZE;
    let _d = build_mode.building_size.tiles_z as f32 * TILE_SIZE;

    if !ui_active && mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(hit_pos) = hit {
            let snapped = snap_building(hit_pos, &build_mode.building_size);
            if check_overlap(&*world_grid, snapped, &build_mode.building_size) {
                return;
            }
            let height = sample_total_height(
                &terrain_sampler,
                &map_generator,
                &sculpt_map,
                Vec3::new(snapped.x, 0.0, snapped.z),
            );
            if let Some(scene) = build_mode
                .placing_building
                .and_then(|bt| building_assets.scene(bt).cloned())
            {
                commands.spawn((
                    Name::new("Building"),
                    SceneRoot(scene),
                    Transform::from_xyz(snapped.x, height, snapped.z),
                ));
            }
            mark_occupied(
                &mut *world_grid,
                &terrain_sampler,
                &map_generator,
                &sculpt_map,
                snapped,
                &build_mode.building_size,
            );
            return;
        }
    }

    let mut ghost_entity: Option<Entity> = None;
    let mut old_size: Option<(u32, u32)> = None;

    for (entity, mut transform, mut visibility, ghost_size, ghost_mats) in ghost_query.iter_mut()
    {
        ghost_entity = Some(entity);
        old_size = Some((ghost_size.tiles_x, ghost_size.tiles_z));

        if ui_active {
            *visibility = Visibility::Hidden;
            continue;
        }

        if let Some(hit_pos) = hit {
            let snapped = snap_building(hit_pos, &build_mode.building_size);
            let height = sample_total_height(
                &terrain_sampler,
                &map_generator,
                &sculpt_map,
                Vec3::new(snapped.x, 0.0, snapped.z),
            );
            transform.translation = Vec3::new(snapped.x, height, snapped.z);
            let overlapping =
                check_overlap(&*world_grid, snapped, &build_mode.building_size);
            if overlapping {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(ghost_mats.red.clone()));
            } else {
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(ghost_mats.gold.clone()));
            }
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    let s = &build_mode.building_size;
    if let Some(entity) = ghost_entity {
        if let Some((old_x, old_z)) = old_size {
            if old_x != s.tiles_x || old_z != s.tiles_z {
                commands.entity(entity).insert(GhostSize {
                    tiles_x: s.tiles_x,
                    tiles_z: s.tiles_z,
                });
            }
        }
    } else {
        let Some(building_type) = build_mode.placing_building else {
            return;
        };
        let Some(mesh_handle) = building_assets.mesh(building_type).cloned() else {
            return;
        };
        let gold_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.85, 0.0, 0.4),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let red_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.2, 0.2, 0.4),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        commands.spawn((
            GhostBuilding,
            GhostSize {
                tiles_x: s.tiles_x,
                tiles_z: s.tiles_z,
            },
            GhostMaterials {
                gold: gold_mat.clone(),
                red: red_mat.clone(),
            },
            Mesh3d(mesh_handle),
            MeshMaterial3d(gold_mat),
            Transform::default(),
            Visibility::Hidden,
        ));
    }
}
