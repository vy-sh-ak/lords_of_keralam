You're right. I stopped midway through the shader implementation. Let me give you the full Bevy implementation plan I'd use if I were building your city-builder terrain system today.

---

# Goal

Move from:

```text
HeightMap
    ↓
ColorMap
    ↓
Texture Image
```

to:

```text
HeightMap
    ↓
Terrain Mesh
    ↓
Custom Terrain Material
    ↓
Grass / Sand / Rock / Water textures
    ↓
Height-based blending
    ↓
Slope-based rock placement
    ↓
Tint support
```

This is essentially Sebastian Lague's terrain shader adapted to modern Bevy.

---

# Phase 1 — Keep HeightMap Generation As-Is

You already have:

```rust
TerrainSampler
NoiseMap
MeshGenerator
Chunk System
LOD System
```

Don't touch any of that.

Keep generating:

```rust
Vec<Vec<f32>>
```

or

```rust
Vec<f32>
```

height values.

The shader will use the final vertex positions.

---

# Phase 2 — Create Terrain Layer Config

Instead of Unity's:

```csharp
Layer[]
```

create:

```rust
#[derive(Clone)]
pub struct TerrainLayer {
    pub start_height: f32,

    pub blend_strength: f32,

    pub tint: LinearRgba,

    pub tint_strength: f32,

    pub texture_scale: f32,

    pub texture: Handle<Image>,
}
```

---

Resource:

```rust
#[derive(Resource)]
pub struct TerrainTextureConfig {
    pub min_height: f32,
    pub max_height: f32,

    pub layers: Vec<TerrainLayer>,
}
```

Example:

```rust
TerrainTextureConfig {
    min_height: 0.0,
    max_height: 100.0,

    layers: vec![
        water,
        sand,
        grass,
        rock,
    ]
}
```

---

# Phase 3 — Create Custom Material

Create:

```text
materials/
 ├── terrain_material.rs
 └── terrain.wgsl
```

---

Material:

```rust
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainMaterial {

    #[uniform(0)]
    pub min_height: f32,

    #[uniform(0)]
    pub max_height: f32,

    #[uniform(0)]
    pub layer_count: u32,

    #[uniform(0)]
    pub start_heights: [f32; 8],

    #[uniform(0)]
    pub blend_strengths: [f32; 8],

    #[uniform(0)]
    pub tint_strengths: [f32; 8],

    #[uniform(0)]
    pub texture_scales: [f32; 8],

    #[uniform(0)]
    pub tints: [Vec4; 8],
}
```

---

# Phase 4 — Start Simple

Do NOT build texture arrays yet.

Bind textures individually.

```rust
#[texture(1)]
#[sampler(2)]
pub grass_texture: Handle<Image>;

#[texture(3)]
#[sampler(4)]
pub rock_texture: Handle<Image>;

#[texture(5)]
#[sampler(6)]
pub sand_texture: Handle<Image>;

#[texture(7)]
#[sampler(8)]
pub water_texture: Handle<Image>;
```

Much easier to debug.

---

# Phase 5 — Use a Material Extension

If using Bevy 0.17:

```rust
MaterialPlugin::<TerrainMaterial>::default()
```

Implement:

```rust
impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }
}
```

---

# Phase 6 — Pass World Position

The shader needs:

```text
world position
world normal
```

for:

```text
height lookup
triplanar mapping
slope detection
```

---

Vertex output:

```wgsl
struct VertexOutput {
    @builtin(position)
    clip_position: vec4<f32>,

    @location(0)
    world_pos: vec3<f32>,

    @location(1)
    world_normal: vec3<f32>,
}
```

---

# Phase 7 — Implement InverseLerp

Unity:

```csharp
inverseLerp()
```

WGSL:

```wgsl
fn inverse_lerp(
    a: f32,
    b: f32,
    value: f32
) -> f32 {

    return clamp(
        (value - a) / (b - a),
        0.0,
        1.0
    );
}
```

---

# Phase 8 — Implement Triplanar Mapping

This is critical.

Without it:

```text
cliffs stretch
```

With it:

```text
cliffs look natural
```

Function:

```wgsl
fn triplanar(
    texture: texture_2d<f32>,
    sampler_tex: sampler,
    world_pos: vec3<f32>,
    scale: f32,
    blend_axes: vec3<f32>,
) -> vec3<f32> {

    let p = world_pos / scale;

    let x =
        textureSample(texture, sampler_tex, p.yz).rgb
        * blend_axes.x;

    let y =
        textureSample(texture, sampler_tex, p.xz).rgb
        * blend_axes.y;

    let z =
        textureSample(texture, sampler_tex, p.xy).rgb
        * blend_axes.z;

    return x + y + z;
}
```

---

# Phase 9 — Calculate Height %

Equivalent to:

```csharp
heightPercent
```

WGSL:

```wgsl
let height_percent =
    inverse_lerp(
        min_height,
        max_height,
        world_pos.y
    );
```

---

# Phase 10 — Calculate Slope

This is where I would improve Sebastian's implementation.

```wgsl
let slope =
    1.0 - world_normal.y;
```

Results:

```text
0.0 = flat ground

1.0 = vertical cliff
```

---

# Phase 11 — Water Layer

```wgsl
if height_percent < 0.20 {
    color = water_texture;
}
```

---

# Phase 12 — Sand Layer

```wgsl
if height_percent > 0.20 &&
   height_percent < 0.30
{
    color = sand_texture;
}
```

---

# Phase 13 — Grass Layer

```wgsl
if height_percent > 0.30 {
    color = grass_texture;
}
```

---

# Phase 14 — Override With Rock On Steep Slopes

This makes a huge difference visually.

```wgsl
if slope > 0.5 {
    color = rock_texture;
}
```

---

# Phase 15 — Smooth Blending

Replace hard transitions.

Instead of:

```wgsl
if grass
```

Use:

```wgsl
smoothstep()
```

Example:

```wgsl
let grass_weight =
    smoothstep(
        0.25,
        0.40,
        height_percent
    );
```

Now textures blend naturally.

---

# Phase 16 — Add Tinting

Unity:

```csharp
baseColour
```

WGSL:

```wgsl
final_color =
    texture_color * (1.0 - tint_strength)
    +
    tint.rgb * tint_strength;
```

Useful for:

```text
summer
autumn
spring
dry season
wet season
```

without changing textures.

---

# Phase 17 — Add Texture Scale

Unity:

```csharp
textureScale
```

WGSL:

```wgsl
let p =
    world_pos / texture_scale;
```

Allows:

```text
grass = dense

rock = large

sand = medium
```

---

# Phase 18 — Convert To Texture Array (Optional)

After everything works:

Replace:

```text
water texture
sand texture
grass texture
rock texture
```

with:

```text
texture_2d_array
```

Exactly like Sebastian's implementation.

Benefits:

* cleaner shader
* arbitrary number of layers
* closer to Unity version

---

# What I'd actually build for your Manor Lords-style project

Version 1:

```text
Height based:
  Water
  Sand
  Grass

Slope based:
  Rock

Triplanar:
  Enabled

Tint:
  Enabled

Texture Array:
  Disabled
```

Version 2:

```text
Texture Array
Biome System
Season Tinting
Distance Blending
```

Get Version 1 working first. The biggest visual upgrade comes from **triplanar mapping + slope-based rock placement**, not from texture arrays.
