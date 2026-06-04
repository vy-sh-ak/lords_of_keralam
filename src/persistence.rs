use bevy_persistent::prelude::*;
use bevy::prelude::Resource;
use serde::{Serialize, de::DeserializeOwned};
use std::env;
use std::path::{Path, PathBuf};

pub struct PersistenceConfig {
    pub save_dir: PathBuf,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        let save_dir = env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kl_lords")
            .join("saves");

        Self { save_dir }
    }
}

impl PersistenceConfig {
    pub fn new(folder_name: &str) -> Self {
        let save_dir = env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            // If not found (unlikely on Windows), safely back up to a "local" directory
            .unwrap_or_else(|| Path::new("local").to_path_buf())
            .join("bevy-persistent")
            .join("kl_lords")
            .join(folder_name);
        println!("Saved data safely to: {:?}", save_dir);
        Self { save_dir }
    }

    pub fn get_resource<R>(
        &self,
        resource_name: &str,
        file_name: impl AsRef<Path>,
    ) -> Persistent<R>
    where
        R: Resource + Default + Serialize + DeserializeOwned,
    {
        Persistent::<R>::builder()
            .name(resource_name)
            .format(StorageFormat::Bincode)
            .path(self.save_dir.join(file_name))
            .default(R::default())
            .build()
            .unwrap_or_else(|error| panic!("failed to initialize {resource_name}: {error}"))
    }
}
