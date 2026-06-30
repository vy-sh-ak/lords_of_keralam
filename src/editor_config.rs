use std::time::Duration;

use bevy::prelude::*;

use bevy_persistent::Persistent;
use serde::{Serialize, de::DeserializeOwned};

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
        let value = self.world().resource::<Persistent<T>>();

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
        let value = self.world().resource::<Persistent<T>>();

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
