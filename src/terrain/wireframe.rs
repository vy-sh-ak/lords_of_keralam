use bevy::{
    pbr::wireframe::{Wireframe, WireframeColor},
    prelude::*,
};

const WIREFRAME_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);

pub fn apply_wireframe_debug(entity_commands: &mut EntityCommands, show_wireframe: bool) {
    if show_wireframe {
        entity_commands.insert((
            Wireframe,
            WireframeColor {
                color: WIREFRAME_COLOR.into(),
            },
        ));
    } else {
        entity_commands.remove::<Wireframe>();
        entity_commands.remove::<WireframeColor>();
    }
}