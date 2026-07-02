use bevy::prelude::*;
use bevy_egui::egui;

use crate::building_placement::BuildMode;

pub fn render_build_mode_panel(world: &mut World, ui: &mut egui::Ui) {
    let mut build_mode = world.resource_mut::<BuildMode>();

    ui.add_space(8.0);
    ui.heading("Building Placement");
    ui.separator();
    ui.checkbox(&mut build_mode.enabled, "Build Mode");
}
