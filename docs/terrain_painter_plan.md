You're at the point where your terrain system needs to transition from **procedural generation** to **player-controlled terrain editing**.

For a city builder, I would strongly recommend **not editing the original noise function directly**. Instead, think of the terrain height as:

```text
Final Height =
    Base Procedural Noise
    + User Sculpt Layer
```

rather than:

```text
Final Height =
    Modified Noise
```

This is how most modern terrain editors work.

---

# Step 1: Split Terrain Into Layers

Currently you probably have:

```rust
height = noise.get(x, z);
```

Change it to:

```rust
height =
    base_noise_height
    + sculpt_height;
```

Store sculpt information separately.

Example:

```rust
pub struct TerrainChunk {
    pub noise_height_map: Vec<Vec<f32>>,
    pub sculpt_map: Vec<Vec<f32>>,
}
```

Then:

```rust
let final_height =
    noise_height_map[y][x]
    + sculpt_map[y][x];
```

Benefits:

* Regenerate terrain anytime
* Save player edits separately
* Undo system becomes easy
* Roads can flatten terrain without destroying noise

---

# Step 2: Create Terrain Brush System

Create a brush:

```rust
pub struct TerrainBrush {
    pub radius: f32,
    pub strength: f32,
}
```

When player clicks:

```rust
brush.paint(
    world_position,
    radius,
    strength,
);
```

---

# Step 3: Raycast Mouse To Terrain

When player holds:

```rust
LMB -> Raise
RMB -> Lower
```

Convert mouse position:

```rust
camera.viewport_to_world(...)
```

Find hit point:

```rust
Vec3 {
    x,
    y,
    z,
}
```

---

# Step 4: Modify Height Values

For every terrain vertex near the brush:

```rust
distance = center.distance(vertex);
```

If:

```rust
distance < radius
```

apply falloff:

```rust
let influence =
    1.0 - distance / radius;
```

Then:

```rust
sculpt_map[y][x] +=
    strength * influence * delta_time;
```

Produces smooth hills.

---

# Step 5: Add Brush Falloff

Hard brushes look terrible.

Use:

```rust
let t = distance / radius;

let falloff =
    (1.0 - t).powf(2.0);
```

or

```rust
smoothstep
```

Example:

```rust
let influence =
    smoothstep(1.0, 0.0, t);
```

Result:

```text
       /\
     /    \
   /        \
```

instead of

```text
   ______
  |      |
  |______|
```

---

# Step 6: Rebuild Chunk Mesh

After editing:

```rust
sculpt_map changed
```

Mark chunk dirty:

```rust
chunk.needs_rebuild = true;
```

Then:

```rust
if chunk.needs_rebuild {
    regenerate_mesh();
}
```

Don't rebuild every chunk.

Only rebuild:

```text
Edited chunk
Neighbor chunks if edge affected
```

---

# Step 7: Add Different Terrain Tools

A city builder typically needs more than raise/lower.

## Raise

```rust
height += strength;
```

---

## Lower

```rust
height -= strength;
```

---

## Flatten

Store target height:

```rust
flatten_height
```

Then:

```rust
height = lerp(
    height,
    flatten_height,
    influence * strength
);
```

Used for:

* city blocks
* plazas
* building lots

---

## Smooth

Average neighboring heights:

```rust
height =
(
 left +
 right +
 up +
 down +
 center
) / 5.0;
```

Essential for removing ugly bumps.

---

## Noise Paint

Add small noise:

```rust
height += noise * strength;
```

Used for natural terrain.

---

# Step 8: Add Road Terraforming

Most city builders do this automatically.

When placing road:

```rust
target_height =
    average_height_along_road;
```

Blend nearby terrain:

```rust
terrain =
    lerp(
        terrain,
        target_height,
        road_falloff
    );
```

Result:

```text
Road
========

Terrain smoothly blends
```

instead of clipping through hills.

---

# Step 9: Store Sculpt Data Per Chunk

For endless terrain:

```rust
HashMap<ChunkCoord, TerrainChunk>
```

Each chunk stores:

```rust
sculpt_map
```

Example:

```rust
pub struct TerrainChunk {
    pub coord: IVec2,
    pub sculpt_map: Vec<f32>,
}
```

Save only sculpt data.

Procedural noise can always be regenerated.

---

# Step 10: Future-Proof for City Builders

Instead of storing only height:

```rust
struct TerrainCell {
    height: f32,
}
```

Store:

```rust
struct TerrainCell {
    base_height: f32,
    sculpt_offset: f32,

    terrain_type: TerrainType,

    fertility: f32,
    moisture: f32,
}
```

Later you'll need:

* grass
* sand
* rock
* road influence
* zoning
* water

without redesigning terrain.

---

# Recommended Architecture

```text
Terrain System
│
├─ Noise Generator
│      Generates base terrain
│
├─ Sculpt Layer
│      Player edits
│
├─ Terrain Mesh Builder
│      Creates mesh
│
├─ Brush Tools
│      Raise
│      Lower
│      Flatten
│      Smooth
│
└─ Save System
       Saves sculpt data only
```

For your Bevy city builder specifically, the next feature after sculpting should be **flatten tool + road terraforming**, because city builders spend far more time creating buildable land than creating mountains. Raise/lower is useful, but flattening terrain to prepare building plots is what players will use constantly.
