use std::any::type_name;
use std::time::Duration;

use bevy::{prelude::*, reflect::GetTypeRegistration};
use bevy_inspector_egui::{
    bevy_inspector::ui_for_value,
    inspector_options::InspectorOptionsType,
};
use bevy_persistent::Persistent;
use egui_dock::egui;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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

#[derive(Resource)]
pub struct EditorState<T: Resource + Clone> {
    pub edited: T,
}
pub trait EditorStateAppExt {
    fn add_editor_state<T>(&mut self) -> &mut Self
    where
        T: Resource + Clone + Serialize + DeserializeOwned;
}

impl EditorStateAppExt for App {
    fn add_editor_state<T>(&mut self) -> &mut Self
    where
        T: Resource + Clone + Serialize + DeserializeOwned,
    {
        let value = self
            .world()
            .resource::<Persistent<T>>();

        let editor_state = EditorState::<T> {
            edited: (**value).clone(),
        };

        self.insert_resource(editor_state);

        self
    }
}

#[derive(Resource)]
pub struct AutosaveState<T: Resource + Clone> {
    pub last_saved: T,
    pub timer: Timer,
}

pub trait AutosaveAppExt {
    fn add_autosave<T>(&mut self) -> &mut Self
    where
        T: Resource + Clone + Serialize + DeserializeOwned + PartialEq;
}

impl AutosaveAppExt for App {
    fn add_autosave<T>(&mut self) -> &mut Self
    where
        T: Resource + Clone + Serialize + DeserializeOwned + PartialEq,
    {
        let value = self
            .world()
            .resource::<Persistent<T>>();

        let state = AutosaveState::<T> {
            last_saved: (**value).clone(),
            timer: Timer::new(Duration::from_secs(4), TimerMode::Repeating),
        };

        self.insert_resource(state);
        self.add_systems(Update, autosave_system::<T>);
        self
    }
}

pub fn autosave_system<T>(
    time: Res<Time>,
    mut autosave: ResMut<AutosaveState<T>>,
    persistent: Res<Persistent<T>>,
) where
    T: Resource + Clone + Serialize + DeserializeOwned + PartialEq,
{
    autosave.timer.tick(time.delta());

    if autosave.timer.just_finished() {
        if &**persistent != &autosave.last_saved {
            persistent.persist().unwrap();
            autosave.last_saved = (**persistent).clone();
        }
    }
}