use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct BuildMode {
    pub enabled: bool,
}
