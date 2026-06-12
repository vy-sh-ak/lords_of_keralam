use std::any::type_name;

use bevy::{prelude::*, reflect::GetTypeRegistration};
use bevy_inspector_egui::{
    bevy_inspector::ui_for_value,
    inspector_options::InspectorOptionsType,
};
use bevy_persistent::Persistent;
use egui_dock::egui;
use serde::{Deserialize, Serialize};

use crate::persistence::PersistenceConfig;

pub trait EditorConfig:
    Resource
    + Reflect
    + InspectorOptionsType
    + GetTypeRegistration
    + Default
    + Serialize
    + for<'de> Deserialize<'de>
{
}

impl<T> EditorConfig for T
where
    T: Resource
        + Reflect
        + InspectorOptionsType
        + GetTypeRegistration
        + Default
        + Serialize
        + for<'de> Deserialize<'de>,
{
}

pub struct EditorConfigInfo {
    pub name: &'static str,
}

#[derive(Resource, Default)]
pub struct EditorRegistry {
    pub configs: Vec<EditorConfigInfo>,
}

pub trait EditorConfigAppExt {
    fn add_editor_config<T>(
        &mut self,
        persistence: &PersistenceConfig,
        resource_name: &'static str,
        file_name: &str,
    ) -> &mut Self
    where
        T: EditorConfig + TypePath;
}

impl EditorConfigAppExt for App {
    fn add_editor_config<T>(
        &mut self,
        persistence: &PersistenceConfig,
        resource_name: &'static str,
        file_name: &str,
    ) -> &mut Self
    where
        T: EditorConfig + TypePath,
    {
        self.register_type::<T>();

        let persistent =
            persistence.get_resource::<T>(resource_name, file_name);

        self.insert_resource(persistent);

        if !self.world().contains_resource::<EditorRegistry>() {
            self.insert_resource(EditorRegistry::default());
        }

        self.world_mut()
            .resource_mut::<EditorRegistry>()
            .configs
            .push(EditorConfigInfo {
                name: type_name::<T>(),
            });

        self
    }
}