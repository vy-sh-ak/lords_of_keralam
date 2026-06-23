use std::fs;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::terrain::MapGenerator;
use crate::terrain_painter::SculptMap;

#[derive(Serialize, Deserialize)]
pub struct MapMetadata {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct MapAsset {
    pub metadata: MapMetadata,
    pub map_generator: MapGenerator,
    pub sculpt_map: SculptMap,
}

#[derive(Resource, Default)]
pub struct ActiveMap {
    pub current_map: Option<String>,
}

pub struct MapAssetService;

impl MapAssetService {
    fn maps_dir() -> PathBuf {
        PathBuf::from("assets").join("maps")
    }

    fn file_path(name: &str) -> PathBuf {
        let sanitized: String = name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        Self::maps_dir().join(format!("{}.json", sanitized))
    }

    pub fn save(asset: &MapAsset) -> Result<(), String> {
        let dir = Self::maps_dir();
        fs::create_dir_all(&dir).map_err(|e| format!("failed to create maps dir: {e}"))?;
        let path = Self::file_path(&asset.metadata.name);
        let json = serde_json::to_string_pretty(asset)
            .map_err(|e| {
                eprintln!("[MapAssetService] serde_json error: {e}");
                format!("failed to serialize map asset: {e}")
            })?;
        fs::write(&path, &json).map_err(|e| format!("failed to write {}: {e}", path.display()))?;
        info!("Saved map asset '{}' to {}", asset.metadata.name, path.display());
        Ok(())
    }

    pub fn load(name: &str) -> Result<MapAsset, String> {
        let path = Self::file_path(name);
        if !path.exists() {
            return Err(format!("map '{}' not found at {}", name, path.display()));
        }
        let json = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let asset: MapAsset = serde_json::from_str(&json)
            .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
        Ok(asset)
    }

    pub fn delete(name: &str) -> Result<(), String> {
        let path = Self::file_path(name);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| format!("failed to delete {}: {e}", path.display()))?;
        }
        Ok(())
    }

    pub fn list() -> Vec<String> {
        let dir = Self::maps_dir();
        if !dir.exists() {
            return Vec::new();
        }
        let mut names = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        names.push(stem.to_string());
                    }
                }
            }
        }
        names.sort();
        names
    }
}
