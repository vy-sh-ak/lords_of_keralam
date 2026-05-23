use bevy::prelude::*;

const TILE_SIZE: f32 = 4.0;
const CHUNK_SIZE: usize = 32;

#[derive(Clone, Copy)]
pub struct Tile {
    pub height: f32,
}

pub struct Chunk {
    pub tiles: Vec<Tile>,
}

pub fn generate_chunk() -> Chunk {
    let mut tiles = Vec::new();

    for _ in 0..(CHUNK_SIZE * CHUNK_SIZE) {
        tiles.push(Tile { height: 0.0 });
    }

    Chunk { tiles }
}

pub fn spawn_chunk_tiles(commands: &mut Commands, chunk: &Chunk, tile_scene: Handle<Scene>) {
    for z in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let tile = chunk.tiles[z * CHUNK_SIZE + x];
            let world_x = x as f32 * TILE_SIZE;
            let world_z = z as f32 * TILE_SIZE;
            commands.spawn((
                Name::new(format!("Tile {x},{z}")),
                SceneRoot(tile_scene.clone()),
                Transform::from_xyz(world_x, tile.height, world_z)
                    .with_scale(Vec3::splat(1.1)),
            ));
        }
    }
}
