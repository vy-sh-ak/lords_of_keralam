
pub mod camera_settings;
pub mod camera_movement;
pub mod camera_sync;
pub mod world_direction;
use bevy::{
    prelude::*,
};
use crate::{editor_config::{AutosaveAppExt, EditorStateAppExt}};
pub use camera_settings::{CameraSettings, DEFAULT_ORBIT_PITCH};
pub use camera_movement::{move_focus, rotate_horizontal, rotate_vertical, zoom};
pub use camera_sync::{clamp_camera_to_grid, sync_world_direction};
pub use world_direction::WorldDirection;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CameraSystems {
    UpdateState,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        let persistence = crate::persistence::PersistenceConfig::new("camera_config");
        let camera_settings = persistence.get_resource::<CameraSettings>(
            "camera settings",
            "camera_settings.toml",
        );
        let world_direction =
            WorldDirection::from_orbit_yaw(camera_settings.orbit_yaw);

        app.insert_resource(camera_settings)
            .insert_resource(world_direction)
            .register_type::<CameraSettings>()
            .add_editor_state::<CameraSettings>()
            .add_autosave::<CameraSettings>()
            .add_systems(
                Update,
                (
                    (rotate_horizontal, rotate_vertical).chain(),
                    move_focus,
                    clamp_camera_to_grid,
                    zoom,
                    sync_world_direction,
                )
                    .chain()
                    .in_set(CameraSystems::UpdateState),
            );
    }
}

