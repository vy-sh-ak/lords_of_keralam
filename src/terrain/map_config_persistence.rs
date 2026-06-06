use bevy::{prelude::*, time::Time};
use bevy_persistent::Persistent;

use super::MapConfigs;

const AUTOSAVE_IDLE_SECONDS: f32 = 5.0;

pub struct MapConfigPersistencePlugin;

#[derive(Resource)]
pub struct MapConfigAutosave {
    timer: Timer,
    dirty: bool,
}

impl Default for MapConfigAutosave {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(AUTOSAVE_IDLE_SECONDS, TimerMode::Once);
        timer.pause();

        Self { timer, dirty: false }
    }
}

impl MapConfigAutosave {
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.timer.reset();
        self.timer.unpause();
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
        self.timer.pause();
    }
}

impl Plugin for MapConfigPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapConfigAutosave::default())
            .add_systems(Update, autosave_map_configs);
    }
}

fn autosave_map_configs(
    time: Res<Time>,
    mut autosave: ResMut<MapConfigAutosave>,
    map_configs: Res<Persistent<MapConfigs>>,
) {
    if !autosave.dirty {
        return;
    }

    autosave.timer.tick(time.delta());
    if !autosave.timer.just_finished() {
        return;
    }

    if let Err(error) = map_configs.persist() {
        bevy::log::error!("failed to autosave map configs: {error}");
        autosave.timer.reset();
        autosave.timer.unpause();
        return;
    }

    autosave.mark_clean();
}