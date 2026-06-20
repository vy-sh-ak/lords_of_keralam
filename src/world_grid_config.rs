use std::collections::HashMap;

use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::camera_config::CameraSettings;
use crate::terrain::{MapGenerator, TerrainSampler};

pub const TILE_SIZE: f32 = 4.0;
pub const GRID_SIZE: usize = 1024;
pub const WORLD_SIZE: f32 = GRID_SIZE as f32 * TILE_SIZE;

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
    pub show_grid: bool,
}

impl Default for WorldGrid {
    fn default() -> Self {
        Self {
            tiles: HashMap::new(),
            show_grid: true,
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
            .add_systems(Update, (update_grid, tile_clicked));
    }
}

fn update_grid(
    mut gizmos: Gizmos,
    mut world_grid: ResMut<WorldGrid>,
    camera_settings: Res<CameraSettings>,
    terrain_sampler: Res<TerrainSampler>,
    map_generator: Res<Persistent<MapGenerator>>,
) {
    if !world_grid.show_grid {
        return;
    }

    let camera_focus = Vec3::new(camera_settings.focus_xz.x, 0.0, camera_settings.focus_xz.y);

    let center_tile = TilePos::world_to_grid(camera_focus);
    let radius = 50.0;

    let min_x = (center_tile.x as f32 - radius).floor().max(0.0) as i32;
    let max_x = (center_tile.x as f32 + radius)
        .ceil()
        .min((GRID_SIZE) as f32) as i32;
    let min_y = (center_tile.y as f32 - radius).floor().max(0.0) as i32;
    let max_y = (center_tile.y as f32 + radius)
        .ceil()
        .min((GRID_SIZE) as f32) as i32;
    let color = Color::srgba(1.0, 1.0, 1.0, 0.2);

    for x in min_x..=max_x {
        let mut points = Vec::new();

        for y in min_y..=max_y {
            let pos = TilePos::new(x, y);

            let height = world_grid
                .ensure_tile(pos, &terrain_sampler, &map_generator)
                .terrain_height;

            points.push(Vec3::new(
                x as f32 * TILE_SIZE,
                height + 0.05,
                y as f32 * TILE_SIZE,
            ));
        }

        gizmos.linestrip(points, color);
    }

    for y in min_y..=max_y {
        let mut points = Vec::new();

        for x in min_x..=max_x {
            let pos = TilePos::new(x, y);

            let height = world_grid
                .ensure_tile(pos, &terrain_sampler, &map_generator)
                .terrain_height;

            points.push(Vec3::new(
                x as f32 * TILE_SIZE,
                height + 0.05,
                y as f32 * TILE_SIZE,
            ));
        }

        gizmos.linestrip(points, color);
    }
    let max_world = (GRID_SIZE - 1) as f32 * TILE_SIZE;

    let mut bottom_points = Vec::new();
    let mut top_points = Vec::new();
    let mut left_points = Vec::new();
    let mut right_points = Vec::new();

    for x in 0..GRID_SIZE as i32 {
        let pos = TilePos::new(x, 0);

        let height = world_grid
            .ensure_tile(pos, &terrain_sampler, &map_generator)
            .terrain_height;

        bottom_points.push(Vec3::new(x as f32 * TILE_SIZE, height + 0.1, 0.0));
    }

    for x in 0..GRID_SIZE as i32 {
        let pos = TilePos::new(x, GRID_SIZE as i32 - 1);

        let height = world_grid
            .ensure_tile(pos, &terrain_sampler, &map_generator)
            .terrain_height;

        top_points.push(Vec3::new(x as f32 * TILE_SIZE, height + 0.1, max_world));
    }

    for y in 0..GRID_SIZE as i32 {
        let pos = TilePos::new(0, y);

        let height = world_grid
            .ensure_tile(pos, &terrain_sampler, &map_generator)
            .terrain_height;

        left_points.push(Vec3::new(0.0, height + 0.1, y as f32 * TILE_SIZE));
    }

    for y in 0..GRID_SIZE as i32 {
        let pos = TilePos::new(GRID_SIZE as i32 - 1, y);

        let height = world_grid
            .ensure_tile(pos, &terrain_sampler, &map_generator)
            .terrain_height;

        right_points.push(Vec3::new(max_world, height + 0.1, y as f32 * TILE_SIZE));
    }

    gizmos.linestrip(bottom_points, Color::srgb(1.0, 0.0, 0.0));
    gizmos.linestrip(top_points, Color::srgb(1.0, 0.0, 0.0));
    gizmos.linestrip(left_points, Color::srgb(1.0, 0.0, 0.0));
    gizmos.linestrip(right_points, Color::srgb(1.0, 0.0, 0.0));

}

fn tile_clicked(
    world_grid: Res<WorldGrid>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let (camera, camera_transform) = camera_query.into_inner();

    if let Some(cursor_pos) = window.cursor_position()
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos)
        && let Some(pos) = ray
            .intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Dir3::Y))
            .map(|distance| ray.get_point(distance))
    {
        let tile_pos = TilePos::world_to_grid(pos);
        if world_grid.tiles.contains_key(&tile_pos) {
            info!("Tile clicked at position: {:?}", tile_pos);
        }
    }
}
