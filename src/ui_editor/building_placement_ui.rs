use bevy::prelude::*;
use bevy_egui::egui;

use crate::building_placement::BuildMode;

pub fn render_build_mode_panel(world: &mut World, ui: &mut egui::Ui) {
    let mut build_mode = world.resource_mut::<BuildMode>();

    ui.add_space(8.0);
    ui.heading("Hut Configuration");
    ui.separator();

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Width (tiles):");
        ui.add(egui::Slider::new(&mut build_mode.building_size.tiles_x, 1..=10));
    });
    ui.horizontal(|ui| {
        ui.label("Length (tiles):");
        ui.add(egui::Slider::new(&mut build_mode.building_size.tiles_z, 1..=10));
    });
    ui.horizontal(|ui| {
        ui.label("Scale:");
        ui.add(egui::Slider::new(&mut build_mode.building_size.scale, 0.5..=3.0));
    });
}
