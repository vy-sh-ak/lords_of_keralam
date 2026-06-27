# Phase 1 — Basic Placement (MVP)

**Goal:** Click to place a cube on the terrain.

### Step 1. Enter Build Mode

Create a resource that stores the current editor state.

```rust
pub enum BuildMode {
    None,
    PlaceBuilding(BuildingType),
}
```

Example:

```
Current Tool
------------
Terrain Sculpt
Terrain Paint
Building Placement
```

Only one tool should be active at a time.

---

### Step 2. Cursor → Terrain Raycast

You already have terrain editing.

Reuse the same logic to get

```
Cursor

     ↓
Terrain

World Position
```

Output:

```rust
pub struct CursorHit {
    world_position: Vec3,
    terrain_normal: Vec3,
}
```

---

### Step 3. Snap to Grid

Convert the cursor position into your world grid.

```
Cursor

(23.7, 0, 19.4)

↓

Grid

(24, 20)
```

Store

```rust
GridPos {
    x,
    z,
}
```

---

### Step 4. Ghost Building

Spawn one transparent building.

```
Cursor

↓

Ghost Building
```

Every frame

```
Ghost Transform

=
Grid Position
+
Terrain Height
```

---

### Step 5. Click to Place

On mouse click

```
Ghost

↓

Real Building Entity
```

Congratulations.

You now have building placement.

---

# Phase 2 — Validation

Now prevent invalid placement.

Each frame

```
Ghost

↓

Validate
```

Checks

✔ Inside map

✔ Not underwater

✔ Terrain slope acceptable

✔ No overlap

✔ Enough space

Return

```rust
enum PlacementState {
    Valid,
    Invalid,
}
```

Ghost color

Green

```
████
```

Red

```
████
```

---

# Phase 3 — Rotation

Press

```
R
```

Rotate

```
0°

90°

180°

270°
```

Update footprint.

---

# Phase 4 — Building Definitions

Instead of cubes

Create

```rust
BuildingDefinition {
    name,
    footprint,
    mesh,
    material,
}
```

Example

```
House

2x2

Farm

6x8

Castle

12x14
```

Now the placement system doesn't know anything about buildings.

It only receives

```
BuildingDefinition
```

---

# Phase 5 — Footprints

Instead of checking one tile

```
██

██
```

Store occupied tiles

```
(5,5)

(5,6)

(6,5)

(6,6)
```

Validation checks every occupied tile.

---

# Phase 6 — Terrain Adaptation

Most city builders don't place buildings on arbitrary terrain.

During validation

Compute

```
Maximum Height

Minimum Height

Difference
```

If

```
Difference

>

Allowed
```

Reject placement.

Later you can flatten terrain automatically.

---

# Phase 7 — Preview Extras

Show

* footprint outline
* occupied cells
* invalid cells
* building facing
* entrance marker

Something like

```
□□□□

■■■■

□□□□
```

instead of only showing the mesh.

---

# Phase 8 — Final Architecture

Eventually I'd split the code into something like

```
building/
│
├── mod.rs
├── building_plugin.rs
├── building_definition.rs
├── building_assets.rs
│
├── placement/
│   ├── mod.rs
│   ├── building_placement.rs
│   ├── placement_preview.rs
│   ├── placement_validation.rs
│   ├── placement_rotation.rs
│   ├── placement_input.rs
│   └── placement_cursor.rs
│
├── construction.rs
├── occupancy.rs
└── spawned_building.rs
```

The nice thing is each file has a single responsibility.

---

# Roadmap (recommended order)

```
✓ Terrain Sculpting
✓ Terrain Painting

⬜ Build Mode
⬜ Cursor Raycast
⬜ Grid Snapping
⬜ Ghost Building
⬜ Place Building

⬜ Placement Validation
⬜ Rotation
⬜ Building Definitions
⬜ Multi-tile Footprints
⬜ Occupancy Grid
⬜ Terrain Flattening
⬜ Preview Overlay
⬜ Roads
⬜ Fences
⬜ Walls
```

## One recommendation specific to your project

From our previous discussions, you're building both an **editor** and a **runtime game**. I'd separate placement logic into reusable layers:

* **Core placement systems**: cursor snapping, validation, occupancy, and spawning. These should work in both the editor and the game.
* **Editor systems**: toolbar integration, unlimited placement, save/load support, debug overlays.
* **Gameplay systems**: resource costs, construction time, worker assignments, and placement restrictions.

That separation will let you use the same `building_placement.rs` logic in your editor to author maps and later in-game when the player constructs buildings, without duplicating code.
