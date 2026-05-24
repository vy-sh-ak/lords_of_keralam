use bevy::prelude::*;

const TILE_SIZE: f32 = 4.0;
const CHUNK_SIZE: usize = 12;

#[derive(Clone, Copy)]
pub struct Tile {
    pub height: f32,
}
#[derive(Resource)]
pub struct Chunk {
    pub tiles: Vec<Tile>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileCoord {
    pub x: usize,
    pub z: usize,
}

impl TileCoord {
    pub const fn new(x: usize, z: usize) -> Self {
        Self { x, z }
    }
}

impl Chunk {
    fn index(coord: TileCoord) -> Option<usize> {
        if coord.x >= CHUNK_SIZE || coord.z >= CHUNK_SIZE {
            return None;
        }
        Some(coord.z * CHUNK_SIZE + coord.x)
    }

    pub fn tile(&self, coord: TileCoord) -> Option<&Tile> {
        let index = Self::index(coord)?;
        self.tiles.get(index)
    }

    pub fn tile_to_world(&self, coord: TileCoord) -> Option<Vec3> {
        let tile = self.tile(coord)?;

        Some(Vec3::new(
            coord.x as f32 * TILE_SIZE,
            tile.height,
            coord.z as f32 * TILE_SIZE,
        ))
    }
}

pub fn generate_chunk() -> Chunk {
    let mut tiles = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);

    for _ in 0..(CHUNK_SIZE * CHUNK_SIZE) {
        tiles.push(Tile { height: 0.0 });
    }

    Chunk { tiles }
}

pub fn spawn_chunk_tiles(commands: &mut Commands, chunk: &Chunk, tile_scene: Handle<Scene>) {
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let coord = TileCoord::new(x, z);
            let world = chunk
                .tile_to_world(coord)
                .expect("generated tile coordinate should always be valid");

            commands.spawn((
                Name::new(format!("Tile {x},{z}")),
                coord,
                SceneRoot(tile_scene.clone()),
                Transform::from_translation(world).with_scale(Vec3::splat(1.05)),
            ));
        }
    }
}

pub fn spawn_hut_at_tile(
    commands: &mut Commands,
    chunk: &Chunk,
    coord: TileCoord,
    hut_scene: Handle<Scene>,
) -> Option<Entity> {
    let world = chunk.tile_to_world(coord)?;
    Some(
        commands
            .spawn((
                Name::new(format!("Hut {},{}", coord.x, coord.z)),
                coord,
                SceneRoot(hut_scene),
                Transform::from_translation(world).with_scale(Vec3::splat(4.0)),
            ))
            .id(),
    )
}
