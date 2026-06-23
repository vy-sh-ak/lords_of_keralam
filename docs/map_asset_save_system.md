# Map Asset Saving System - Implementation Plan

## Goal

Implement a Map Asset saving system for the editor.

This system is intended for game developers/designers to create, edit, save, and load terrain maps.

This is NOT a player save-game system.

The purpose is to preserve:

* Terrain generation settings
* Terrain sculpting modifications
* Terrain painting data (future-ready)
* Map metadata

and allow designers to continue editing maps later.

---

# Current Editor Layout

Add a new editor tool button:

```text
[Map Gen] [Terrain Paint] [Map Data]
```

When clicked, it opens the existing right-side editor panel.

Add:

```rust
enum RightPanelKind {
    MapGenerator,
    TerrainPainter,
    MapData,
    Inspector,
}
```

---

# Scope (Build Now)

Implement:

* Create map asset
* Save map asset
* Load map asset
* Delete map asset
* List available map assets

Do NOT implement:

* Player save games
* Buildings
* Roads
* Economy state
* Units
* Runtime simulation state

---

# Map Asset Structure

Create:

```rust
#[derive(Serialize, Deserialize)]
pub struct MapAsset {
    pub metadata: MapMetadata,

    pub map_generator: MapGenerator,

    pub sculpt_map: SculptMap,
}
```

Metadata:

```rust
#[derive(Serialize, Deserialize)]
pub struct MapMetadata {
    pub name: String,
}
```

Only save editor terrain data.

Do not save generated meshes.

Do not save generated noise maps.

Do not save chunk meshes.

Generated terrain must always be recreated from saved data.

---

# Save Folder Structure

Create:

```text
assets/
└── maps/
```

Example:

```text
assets/maps/grasslands.json
assets/maps/desert.json
assets/maps/island.json
```

Use JSON initially for easier debugging.

---

# Active Map Tracking

Create:

```rust
#[derive(Resource)]
pub struct ActiveMap {
    pub current_map: Option<String>,
}
```

Examples:

```rust
None
```

No map loaded.

```rust
Some("grasslands".to_string())
```

Grasslands currently loaded.

---

# Map Asset Service

Create a dedicated service:

```rust
pub struct MapAssetService;
```

Responsibilities:

### Save

```rust
fn save(asset: &MapAsset)
```

Creates or overwrites:

```text
assets/maps/{name}.json
```

### Load

```rust
fn load(name: &str) -> Result<MapAsset>
```

### Delete

```rust
fn delete(name: &str)
```

### List

```rust
fn list() -> Vec<String>
```

Returns all map names.

---

# Maps Editor Panel

New right-side panel:

```text
Maps
--------------------------------

Map Name
[___________________]

[ Create New ]

--------------------------------

Current Map

Grasslands

[ Save ]

--------------------------------

Available Maps

- Grasslands
- Desert
- Island

[ Load ]
[ Delete ]
```

---

# Create New Flow

User enters:

```text
Island
```

Clicks:

```text
Create New
```

System:

1. Creates new MapAsset
2. Copies current editor state
3. Saves file
4. Sets ActiveMap = "Island"
5. Refreshes map list

---

# Save Flow

When Save is clicked:

1. Gather current editor state
2. Build MapAsset
3. Save to disk
4. Overwrite existing file

Current editor state consists of:

```rust
MapGenerator
SculptMap
```

---

# Load Flow

When Load is clicked:

1. Read selected asset
2. Replace Persistent<MapGenerator>
3. Replace SculptMap
4. Trigger terrain regeneration
5. Set ActiveMap

Result:

Terrain should fully rebuild from loaded data.

---

# Delete Flow

When Delete is clicked:

1. Remove file
2. Refresh map list
3. If deleted map was active:

   * Clear ActiveMap

No confirmation dialog required initially.

---

# Terrain Regeneration Requirements

Loading a map must rebuild:

* Height map
* Terrain chunks
* UV mesh
* Grid alignment
* Sculpted terrain

Reuse existing regeneration pipeline.

Do not add a separate loading pipeline.

Loading should behave similarly to changing MapGenerator settings and forcing a full terrain rebuild.

---

# Serialization Requirements

All saved structures must derive:

```rust
Serialize
Deserialize
```

Ensure:

```rust
MapGenerator
SculptMap
MapMetadata
MapAsset
```

can serialize cleanly.

If SculptMap contains runtime-only data, separate runtime data from persistent data before serialization.

---

# Future Considerations (Do Not Build Yet)

This system is intentionally only for map assets.

Later we will introduce:

```rust
WorldSave
```

for player saves.

Planned architecture:

```text
assets/
└── maps/
    grasslands.json
    desert.json

saves/
└── player/
    save_01.json
    save_02.json
```

Map Asset:

```rust
MapGenerator
SculptMap
TerrainPaintData
```

Player Save:

```rust
Map Reference
Buildings
Roads
Units
Resources
Economy
Simulation State
```

Player saves should reference a map asset instead of duplicating terrain data.

Do not implement any player save logic in this task.

---

# Success Criteria

A developer can:

1. Generate terrain.
2. Sculpt terrain.
3. Create a named map.
4. Save the map.
5. Restart the editor.
6. Load the map.
7. Get the exact same terrain back.

No player save functionality is included.

Few more things to consider:
- We can remove the existing persistent MapGenerator resource and instead use the MapAsset system to load and save the MapGenerator settings.
- This change should not break existing functionalities.