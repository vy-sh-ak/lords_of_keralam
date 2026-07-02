# Project conventions

## Plugin architecture
- Each feature module should provide its own `Plugin` struct
- Register resources, persistence, and systems inside the plugin's `build()` method
- Keep `main.rs` lean — only add_plugins and top-level setup
- Follow the pattern in `src/camera_config.rs::CameraPlugin` and `src/map_asset.rs::MapAssetPlugin`

## Persistence
- Use `bevy_persistent` with `PersistenceConfig` for user settings
- Use `editor_config::AutosaveAppExt::add_autosave<T>()` for auto-saving (every 4s)
- Store per-feature resources under `%LOCALAPPDATA%\bevy-persistent\kl_lords\<feature_name>\`

## UI panels
- UI rendering goes in `src/ui_editor/` as egui functions
- Access world state via `world.resource::<T>()` / `world.resource_mut::<T>()`
- Groups of related buttons should use `ui.horizontal()` for compact layout

## Code style
- No comments in code unless absolutely necessary
- Match existing naming conventions (PascalCase for types, snake_case for functions/vars)
