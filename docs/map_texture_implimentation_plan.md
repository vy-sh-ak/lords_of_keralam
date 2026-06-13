Since you're building a **city builder**, I'd skip the pure-color approach entirely and go straight to a **texture splat shader**.

Sebastian Lague's shader is great for learning, but once you start placing roads, farms, forests, and cities, solid colors quickly look dated.

A better progression is:

```text
Noise Map
    ↓
Terrain Mesh
    ↓
Height-based texture blending
    ↓
Biome-based texture blending
    ↓
Road decals / terrain painting
```

---

# Goal

Render:

```text
0.00 - 0.20  -> Sand texture
0.20 - 0.60  -> Grass texture
0.60 - 0.80  -> Rock texture
0.80 - 1.00  -> Snow texture
```

with smooth blending.

Result:

```text
sand ===== grass ===== rock ===== snow
         smooth      smooth
```

instead of

```text
sand | grass | rock | snow
```

---

# Step 1: Create a custom material

Create:

```rust
// terrain_material.rs
```

```rust
use bevy::{
    asset::Asset,
    pbr::Material,
    prelude::*,
    reflect::TypePath,
    render::render_resource::*,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub min_height: f32,

    #[uniform(0)]
    pub max_height: f32,

    #[uniform(0)]
    pub blend_strength: f32,

    #[texture(1)]
    #[sampler(2)]
    pub sand_texture: Handle<Image>,

    #[texture(3)]
    #[sampler(4)]
    pub grass_texture: Handle<Image>,

    #[texture(5)]
    #[sampler(6)]
    pub rock_texture: Handle<Image>,

    #[texture(7)]
    #[sampler(8)]
    pub snow_texture: Handle<Image>,
}
```

---

# Step 2: Register material plugin

```rust
app.add_plugins(MaterialPlugin::<TerrainMaterial>::default());
```

---

# Step 3: Material implementation

```rust
impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_material.wgsl".into()
    }
}
```

---

# Step 4: Add terrain textures

Example:

```text
assets/
 ├─ textures/
 │   ├─ sand.png
 │   ├─ grass.png
 │   ├─ rock.png
 │   └─ snow.png
```

---

# Step 5: Load textures

```rust
let sand = asset_server.load("textures/sand.png");
let grass = asset_server.load("textures/grass.png");
let rock = asset_server.load("textures/rock.png");
let snow = asset_server.load("textures/snow.png");
```

---

# Step 6: Create material

```rust
let terrain_material = materials.add(TerrainMaterial {
    min_height: -20.0,
    max_height: 120.0,

    blend_strength: 0.05,

    sand_texture: sand,
    grass_texture: grass,
    rock_texture: rock,
    snow_texture: snow,
});
```

---

# Step 7: Apply material

```rust
commands.spawn((
    Mesh3d(mesh_handle),
    MeshMaterial3d(terrain_material),
));
```

---

# Step 8: Terrain shader

Create:

```text
assets/shaders/terrain.wgsl
```

---

## Material struct

```wgsl
struct TerrainMaterial {
    min_height: f32,
    max_height: f32,
    blend_strength: f32,
}
```

---

## Bindings

```wgsl
@group(2) @binding(0)
var<uniform> material: TerrainMaterial;

@group(2) @binding(1)
var sand_tex: texture_2d<f32>;

@group(2) @binding(2)
var sand_sampler: sampler;

@group(2) @binding(3)
var grass_tex: texture_2d<f32>;

@group(2) @binding(4)
var grass_sampler: sampler;

@group(2) @binding(5)
var rock_tex: texture_2d<f32>;

@group(2) @binding(6)
var rock_sampler: sampler;

@group(2) @binding(7)
var snow_tex: texture_2d<f32>;

@group(2) @binding(8)
var snow_sampler: sampler;
```

---

## Utility functions

```wgsl
fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    return clamp(
        (value - a) / (b - a),
        0.0,
        1.0
    );
}
```

---

## Fragment shader

```wgsl
@fragment
fn fragment(
    mesh: bevy_pbr::MeshFragmentInput
) -> @location(0) vec4<f32> {

    let world_pos = mesh.world_position;

    let uv = world_pos.xz * 0.05;

    let sand =
        textureSample(sand_tex, sand_sampler, uv);

    let grass =
        textureSample(grass_tex, grass_sampler, uv);

    let rock =
        textureSample(rock_tex, rock_sampler, uv);

    let snow =
        textureSample(snow_tex, snow_sampler, uv);

    let h = inverse_lerp(
        material.min_height,
        material.max_height,
        world_pos.y
    );

    let sand_grass =
        smoothstep(0.15, 0.25, h);

    let grass_rock =
        smoothstep(0.55, 0.65, h);

    let rock_snow =
        smoothstep(0.75, 0.85, h);

    var color =
        mix(sand, grass, sand_grass);

    color =
        mix(color, rock, grass_rock);

    color =
        mix(color, snow, rock_snow);

    return color;
}
```

---

# Step 9: Improve texture tiling

Large worlds look terrible when textures stretch.

Use world-space UVs:

```wgsl
let uv = world_pos.xz * 0.1;
```

instead of mesh UVs.

This is called **triplanar/world projection**.

---

# Step 10: Future upgrade (recommended)

After you get this working:

Add:

```rust
forest_texture
mud_texture
road_texture
farm_texture
stone_texture
```

Then drive texture selection by:

```text
height
+
slope
+
biome
```

For example:

```text
Height < 0.2 → sand

Slope > 45° → rock

Biome == forest → forest texture

Biome == farmland → dirt texture
```

This is much closer to how games like Manor Lords and Cities: Skylines build terrain materials.

For your Bevy city-builder, I'd stop at **height + slope blending** first. It's only about 50 extra lines in WGSL and gives a massive visual improvement over height-only texturing.
