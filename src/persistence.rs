use bevy_persistent::prelude::*;
use bevy::prelude::Resource;
use serde::{Serialize, de::DeserializeOwned};
use std::fs;
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
        let path = self.save_dir.join(file_name);
        let build_resource = || {
            Persistent::<R>::builder()
                .name(resource_name)
                .format(StorageFormat::Bincode)
                .path(&path)
                .default(R::default())
                .build()
        };

        build_resource().unwrap_or_else(|initial_error| {
            eprintln!(
                "failed to initialize {resource_name} from {:?}: {initial_error}",
                path
            );

            backup_unreadable_file(&path);

            build_resource().unwrap_or_else(|retry_error| {
                panic!(
                    "failed to initialize {resource_name} from {:?} after resetting persisted data: {retry_error}",
                    path
                )
            })
        })
    }
}

fn backup_unreadable_file(path: &Path) {
    if !path.exists() {
        return;
    }

    let backup_path = unreadable_backup_path(path);

    if backup_path.exists() {
        let _ = fs::remove_file(&backup_path);
    }

    match fs::rename(path, &backup_path) {
        Ok(()) => {
            eprintln!(
                "moved unreadable persisted data from {:?} to {:?}",
                path, backup_path
            );
        }
        Err(rename_error) => {
            eprintln!(
                "failed to move unreadable persisted data from {:?}: {rename_error}; removing file instead",
                path
            );

            if let Err(remove_error) = fs::remove_file(path) {
                eprintln!(
                    "failed to remove unreadable persisted data at {:?}: {remove_error}",
                    path
                );
            }
        }
    }
}

fn unreadable_backup_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| format!("{}.unreadable", name.to_string_lossy()))
        .unwrap_or_else(|| "persisted-data.unreadable".to_string());

    path.with_file_name(file_name)
}
