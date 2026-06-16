use std::collections::HashMap;

use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::camera_plugin::CameraSettings;
use crate::terrain::{MapGenerator, TerrainSampler};

pub const TILE_SIZE: f32 = 4.0;
pub const GRID_SIZE: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
}

impl TilePos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn grid_to_world(&self) -> Vec3 {
        Vec3::new(self.x as f32 * TILE_SIZE, 0.0, self.y as f32 * TILE_SIZE)
    }

    pub fn world_to_grid(pos: Vec3) -> Self {
        Self {
            x: (pos.x / TILE_SIZE).floor() as i32,
            y: (pos.z / TILE_SIZE).floor() as i32,
        }
    }
}

#[derive(Clone)]
pub struct Tile {
    pub terrain_height: f32,
    pub occupied: bool,
}

#[derive(Resource)]
pub struct WorldGrid {
    pub tiles: HashMap<TilePos, Tile>,
}

impl Default for WorldGrid {
    fn default() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }
}

impl WorldGrid {
    pub fn is_valid(&self, pos: TilePos) -> bool {
        pos.x >= 0 && pos.x < GRID_SIZE as i32 && pos.y >= 0 && pos.y < GRID_SIZE as i32
    }

    pub fn ensure_tile(
        &mut self,
        pos: TilePos,
        terrain_sampler: &TerrainSampler,
        map_generator: &MapGenerator,
    ) -> &mut Tile {
        self.tiles.entry(pos).or_insert_with(|| {
            let world = pos.grid_to_world();
            let height = terrain_sampler.sample_height(map_generator, world.x, world.z);
            Tile {
                terrain_height: height,
                occupied: false,
            }
        })
    }

    pub fn get_tile(&self, pos: TilePos) -> Option<&Tile> {
        self.tiles.get(&pos)
    }
}

pub struct GridGeneratorPlugin;
impl Plugin for GridGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldGrid::default())
            .add_systems(Update, update_grid);
    }
}

fn update_grid(
    mut gizmos: Gizmos,
    mut world_grid: ResMut<WorldGrid>,
    camera_transform: Single<&Transform, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    terrain_sampler: Res<TerrainSampler>,
    map_generator: Res<Persistent<MapGenerator>>,
) {
    let focus = Vec3::new(
        camera_settings.focus_xz.x,
        0.0,
        camera_settings.focus_xz.y,
    );
    let distance = camera_transform.translation.distance(focus);
    let visible_radius = (distance * 0.6).clamp(30.0, 250.0);

    let min_tx = ((camera_settings.focus_xz.x - visible_radius) / TILE_SIZE)
        .floor()
        .max(0.0) as i32;
    let max_tx = ((camera_settings.focus_xz.x + visible_radius) / TILE_SIZE)
        .ceil()
        .min((GRID_SIZE - 1) as f32) as i32;
    let min_tz = ((camera_settings.focus_xz.y - visible_radius) / TILE_SIZE)
        .floor()
        .max(0.0) as i32;
    let max_tz = ((camera_settings.focus_xz.y + visible_radius) / TILE_SIZE)
        .ceil()
        .min((GRID_SIZE - 1) as f32) as i32;

    // Ensure tiles within the visible range exist (lazy init)
    for tx in min_tx..=max_tx {
        for tz in min_tz..=max_tz {
            world_grid.ensure_tile(
                TilePos::new(tx, tz),
                &terrain_sampler,
                &map_generator,
            );
        }
    }

    let color = Color::srgba(1.0, 1.0, 1.0, 0.2);
    let y_offset = 0.05;

    for tz in min_tz..=max_tz {
        let wz = tz as f32 * TILE_SIZE;
        let points: Vec<Vec3> = (min_tx..=max_tx)
            .map(|tx| {
                let tile = &world_grid.tiles[&TilePos::new(tx, tz)];
                Vec3::new(tx as f32 * TILE_SIZE, tile.terrain_height + y_offset, wz)
            })
            .collect();
        gizmos.linestrip(points, color);
    }

    for tx in min_tx..=max_tx {
        let wx = tx as f32 * TILE_SIZE;
        let points: Vec<Vec3> = (min_tz..=max_tz)
            .map(|tz| {
                let tile = &world_grid.tiles[&TilePos::new(tx, tz)];
                Vec3::new(wx, tile.terrain_height + y_offset, tz as f32 * TILE_SIZE)
            })
            .collect();
        gizmos.linestrip(points, color);
    }
}
